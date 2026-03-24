use crate::image_provider::ImageProvider;
use crate::models::Printing;
use image::ImageFormat;
use krilla::Data;
use krilla::Document;
use krilla::color::rgb;
use krilla::geom::{Path, PathBuilder, Size, Transform};
use krilla::image::Image;
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::Stroke;
use std::collections::HashMap;
use tracing::info;
use web_time::Instant;

const POINTS_PER_INCH: f32 = 72.0;

const LETTER_WIDTH: f32 = 8.5 * POINTS_PER_INCH; // 612 points
const LETTER_HEIGHT: f32 = 11.0 * POINTS_PER_INCH; // 792 points
const A4_WIDTH: f32 = 8.27 * POINTS_PER_INCH; // ~595 points
const A4_HEIGHT: f32 = 11.69 * POINTS_PER_INCH; // ~842 points

const CARD_WIDTH: f32 = 178.54; // 6.299 cm in points
const CARD_HEIGHT: f32 = 249.09; // 8.788 cm in points

const MINIMUM_MARGIN: f32 = 0.25 * POINTS_PER_INCH;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum PageSize {
    #[default]
    Letter,
    A4,
    // Custom(f32, f32),
}

#[derive(Clone, Copy, Debug, Default)]
pub enum CutLines {
    #[default]
    None,
    Margins,
    FullPage,
}

// #[derive(Clone, Copy, Debug, Default)]
// pub enum Spacing {
//     #[default]
//     None,
//     Margins,
//     FullPage,
// }

impl PageSize {
    fn dimensions(&self) -> (f32, f32) {
        match self {
            PageSize::Letter => (LETTER_WIDTH, LETTER_HEIGHT),
            PageSize::A4 => (A4_WIDTH, A4_HEIGHT),
            // PageSize::Custom(width, height) => (width * POINTS_PER_INCH, height * POINTS_PER_INCH),
        }
    }

    fn capacity(&self) -> (usize, usize) {
        let (page_width, page_height) = self.dimensions();
        let max_cards_per_row =
            ((page_width - (MINIMUM_MARGIN * 2.0)) / CARD_WIDTH).floor() as usize;
        let max_cards_per_column =
            ((page_height - (MINIMUM_MARGIN * 2.0)) / CARD_HEIGHT).floor() as usize;
        (max_cards_per_column, max_cards_per_row)
    }

    fn margins(&self) -> (f32, f32) {
        let (page_width, page_height) = self.dimensions();
        let (max_cards_per_column, max_cards_per_row) = self.capacity();

        let left_margin = (page_width - ((max_cards_per_column as f32) * CARD_WIDTH)) / 2.0;
        let top_margin = (page_height - ((max_cards_per_row as f32) * CARD_HEIGHT)) / 2.0;

        (left_margin, top_margin)
    }
}

pub async fn generate_pdf(
    printings: Vec<Printing>,
    image_provider: &impl ImageProvider,
    page_size: PageSize,
    cut_lines: CutLines,
    // spacing: Spacing,
    progress: Option<Box<dyn Fn(f32) + Send + Sync>>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let total_images: usize = printings.iter().map(|p| 1 + p.parts.len()).sum();
    let mut processed_images: usize = 0;

    let mut image_keys: Vec<String> = Vec::with_capacity(total_images);
    for p in &printings {
        image_keys.push(p.image_key.clone());
        for part in &p.parts {
            image_keys.push(part.image_key.clone());
        }
    }

    let mut image_cache: HashMap<String, Image> = HashMap::new();
    let mut document = Document::new();
    let (page_width, page_height) = page_size.dimensions();

    let (max_cards_per_column, max_cards_per_row) = page_size.capacity();
    let max_cards_per_page = max_cards_per_column * max_cards_per_row;

    for chunk in image_keys.chunks(max_cards_per_page) {
        let page_settings = PageSettings::from_wh(page_width, page_height).unwrap();
        let mut page = document.start_page_with(page_settings);
        let mut surface = page.surface();

        for (index, image_key) in chunk.iter().enumerate() {
            let start = Instant::now();

            if !image_cache.contains_key(image_key) {
                let image_data = image_provider.get_image_bytes(image_key).await?;
                let format = image::guess_format(&image_data).unwrap_or(ImageFormat::Jpeg);

                let image = if format == ImageFormat::Png {
                    Image::from_png(Data::from(image_data), true)?
                } else {
                    Image::from_jpeg(Data::from(image_data), true)?
                };

                image_cache.insert(image_key.clone(), image);
            }

            let image = image_cache.get(image_key).unwrap().clone();
            let size = Size::from_wh(CARD_WIDTH, CARD_HEIGHT).unwrap();

            let (pos_x, pos_y) = calculate_card_position(index, &page_size);

            surface.push_transform(&Transform::from_translate(pos_x, pos_y));
            surface.draw_image(image, size);
            surface.pop();

            processed_images += 1;
            if let Some(ref cb) = progress
                && total_images > 0
            {
                cb(processed_images as f32 / total_images as f32);
            }

            #[cfg(not(target_arch = "wasm32"))]
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            #[cfg(target_arch = "wasm32")]
            gloo_timers::future::TimeoutFuture::new(0).await;

            info!("Runtime for image {}: {:?}", image_key, start.elapsed());
        }

        surface.set_stroke(Some(Stroke {
            paint: rgb::Color::new(16, 16, 16).into(),
            width: 0.5,
            miter_limit: 0.0,
            line_cap: Default::default(),
            line_join: Default::default(),
            opacity: NormalizedF32::new(1.0).unwrap(),
            dash: None,
        }));

        let lines = match cut_lines {
            CutLines::None => Vec::new(),
            CutLines::Margins => calculate_margin_cutlines(page_size),
            CutLines::FullPage => calculate_full_page_cutlines(page_size),
        };

        for line in &lines {
            surface.draw_path(line);
        }

        surface.finish();
        page.finish();
    }

    let pdf = document.finish().unwrap();
    Ok(pdf)
}

fn calculate_card_position(card_index: usize, page_size: &PageSize) -> (f32, f32) {
    let (left_margin, top_margin) = page_size.margins();

    let col = card_index % 3;
    let row = card_index / 3;

    let x = left_margin + (col as f32 * CARD_WIDTH);
    let y = top_margin + (row as f32 * CARD_HEIGHT);

    (x, y)
}

fn calculate_margin_cutlines(page_size: PageSize) -> Vec<Path> {
    let (left_margin, top_margin) = page_size.margins();
    let line_length: f32 = 15.0;
    let line_gap: f32 = 3.0;

    let mut lines = Vec::<Path>::new();

    // top cut lines
    for i in 0..4 {
        lines.push({
            let mut pb = PathBuilder::new();
            pb.move_to(
                left_margin + (CARD_WIDTH * (i as f32)),
                top_margin - line_length - line_gap,
            );
            pb.line_to(
                left_margin + (CARD_WIDTH * (i as f32)),
                top_margin - line_gap,
            );
            pb.finish().unwrap()
        });
    }

    // bottom cut lines
    for i in 0..4 {
        lines.push({
            let mut pb = PathBuilder::new();
            pb.move_to(
                left_margin + (CARD_WIDTH * (i as f32)),
                top_margin + (CARD_HEIGHT * 3.0) + line_length + line_gap,
            );
            pb.line_to(
                left_margin + (CARD_WIDTH * (i as f32)),
                top_margin + (CARD_HEIGHT * 3.0) + line_gap,
            );
            pb.finish().unwrap()
        });
    }

    // left cut lines
    for i in 0..4 {
        lines.push({
            let mut pb = PathBuilder::new();
            pb.move_to(
                left_margin - line_length - line_gap,
                top_margin + (CARD_HEIGHT * (i as f32)),
            );
            pb.line_to(
                left_margin - line_gap,
                top_margin + (CARD_HEIGHT * (i as f32)),
            );
            pb.finish().unwrap()
        });
    }

    // right cut lines
    for i in 0..4 {
        lines.push({
            let mut pb = PathBuilder::new();
            pb.move_to(
                left_margin + (CARD_WIDTH * 3.0) + line_length + line_gap,
                top_margin + (CARD_HEIGHT * (i as f32)),
            );
            pb.line_to(
                left_margin + (CARD_WIDTH * 3.0) + line_gap,
                top_margin + (CARD_HEIGHT * (i as f32)),
            );
            pb.finish().unwrap()
        });
    }

    lines
}

fn calculate_full_page_cutlines(page_size: PageSize) -> Vec<Path> {
    let (left_margin, top_margin) = page_size.margins();
    let (page_width, page_height) = page_size.dimensions();

    let mut lines = Vec::<Path>::new();

    for i in 0..4 {
        lines.push({
            let mut pb = PathBuilder::new();
            pb.move_to(left_margin + CARD_WIDTH * (i as f32), 0.0);
            pb.line_to(left_margin + CARD_WIDTH * (i as f32), page_height);
            pb.finish().unwrap()
        });
    }

    for i in 0..4 {
        lines.push({
            let mut pb = PathBuilder::new();
            pb.move_to(0.0, top_margin + CARD_HEIGHT * (i as f32));
            pb.line_to(page_width, top_margin + CARD_HEIGHT * (i as f32));
            pb.finish().unwrap()
        });
    }

    lines
}
