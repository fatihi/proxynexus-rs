use crate::analytics;
use crate::components::source_selector::ActiveSource;
use dioxus::prelude::*;
use proxynexus_core::card_source::{Cardlist, NrdbUrl, SetName};
use proxynexus_core::db_storage::DbStorage;
use proxynexus_core::pdf::{PageSize, generate_pdf};
use tracing::{error, info};
use web_time::Instant;

pub async fn run_pdf_export(
    mut db_signal: Signal<DbStorage>,
    active_source: ActiveSource,
    page_size: PageSize,
) {
    analytics::start_capture();
    let start_time = Instant::now();
    let page_size_str = format!("{:?}", page_size);
    info!("Starting PDF generation with page size: {}", page_size_str);

    let mut db = db_signal.write();

    #[cfg(not(target_arch = "wasm32"))]
    let provider = {
        let home = dirs::home_dir().expect("Could not find home directory");
        let collections_path = home.join(".proxynexus").join("collections");
        proxynexus_core::image_provider::LocalImageProvider::new(collections_path)
    };

    #[cfg(target_arch = "wasm32")]
    let provider = proxynexus_core::image_provider::RemoteImageProvider;

    let (result, source_text, source_type) = match active_source {
        ActiveSource::Cardlist(text) => (
            generate_pdf(&mut db, &Cardlist(text.clone()), &provider, page_size).await,
            text,
            "Cardlist",
        ),
        ActiveSource::SetName(name) => (
            generate_pdf(&mut db, &SetName(name.clone()), &provider, page_size).await,
            name,
            "SetName",
        ),
        ActiveSource::NrdbUrl(url) => (
            generate_pdf(&mut db, &NrdbUrl(url.clone()), &provider, page_size).await,
            url,
            "NrdbUrl",
        ),
    };

    let mut success = false;
    let mut error_message = None;

    match result {
        Ok(pdf_bytes) => {
            info!(
                "Successfully generated PDF. Size: {} bytes",
                pdf_bytes.len()
            );

            if let Err(e) = save_pdf(&pdf_bytes).await {
                let msg = format!("Failed to save PDF: {:?}", e);
                error!("{}", msg);
                error_message = Some(msg);
            } else {
                success = true;
            }
        }
        Err(e) => {
            let msg = format!("Failed to generate PDF: {}", e);
            error!("{}", msg);
            error_message = Some(msg);
        }
    }

    analytics::send_report(analytics::GenerationReport {
        format: "pdf".to_string(),
        page_size: page_size_str,
        runtime_ms: start_time.elapsed().as_millis(),
        success,
        source_type,
        source_text,
        error_message,
    });
}

#[cfg(not(target_arch = "wasm32"))]
async fn save_pdf(bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(path) = rfd::AsyncFileDialog::new()
        .add_filter("PDF Document", &["pdf"])
        .set_file_name("proxynexus_export.pdf")
        .save_file()
        .await
    {
        tokio::fs::write(path.path(), bytes).await?;
        info!("Saved PDF successfully to {:?}", path.path());
    } else {
        info!("User cancelled the save dialog.");
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
async fn save_pdf(bytes: &[u8]) -> Result<(), wasm_bindgen::JsValue> {
    // Using native browser APIs to create a Blob directly from WASM memory.
    // This avoids JSON serialization overhead of Dioxus eval, which causes overflow errors on large PDFs
    use wasm_bindgen::JsCast;

    let uint8_array = js_sys::Uint8Array::from(bytes);
    let parts = js_sys::Array::of1(&uint8_array);

    let options = web_sys::BlobPropertyBag::new();
    options.set_type("application/pdf");

    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &options)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob)?;

    let window = web_sys::window().ok_or_else(|| wasm_bindgen::JsValue::from_str("No window"))?;
    let document = window
        .document()
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("No document"))?;

    let a = document
        .create_element("a")?
        .dyn_into::<web_sys::HtmlElement>()?;

    a.set_attribute("href", &url)?;
    a.set_attribute("download", "proxynexus_result.pdf")?;
    a.click();

    web_sys::Url::revoke_object_url(&url)?;

    Ok(())
}
