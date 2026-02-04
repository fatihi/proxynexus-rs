use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    image_path: PathBuf,

    #[arg(short, long, default_value = "output.pdf")]
    output_path: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("Adding {:?} to {:?}!", args.image_path, args.output_path);

    match proxynexus_core::create_pdf_with_image(&args.image_path, &args.output_path) {
        Ok(_) => println!("PDF created successfully: {:?}", &args.output_path),
        Err(e) => eprintln!("Error: {}", e),
    }
}
