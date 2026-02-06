use std::io;
use std::path::PathBuf;
use clap::Parser;
use proxynexus_core::PageSize;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    images_path: PathBuf,

    #[arg(short, long, default_value = "output.pdf")]
    output_path: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("Adding {:?} to {:?}!", args.images_path, args.output_path);

    let images_path = std::fs::read_dir(&args.images_path)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();

    match proxynexus_core::create_pdf_with_images(images_path, &args.output_path, PageSize::Letter) {
        Ok(_) => println!("PDF created successfully: {:?}", &args.output_path),
        Err(e) => eprintln!("Error: {}", e),
    }
}
