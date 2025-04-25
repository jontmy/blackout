use anyhow::Context;
use clap::Parser;
use pdfium_render::prelude::*;
use std::path::PathBuf;
use tqdm::pbar;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: PathBuf,
    #[arg(short, long)]
    output: PathBuf,
    #[arg(short, long)]
    password: Option<String>,
}

fn main() {
    let args = Args::parse();

    if !(args.input.is_file()) {
        eprintln!("Input path (-i) must be a file: {}", args.input.display());
        std::process::exit(1);
    }
    if !args.input.exists() {
        eprintln!("Input file (-i) does not exist: {}", args.input.display());
        std::process::exit(1);
    }
    if let Some("pdf") = args.input.extension().and_then(|s| s.to_str()) {
    } else {
        eprintln!(
            "Input file (-i) must be a PDF file (extension .pdf): {}",
            args.input.display()
        );
        std::process::exit(1);
    }

    if args.output.exists() && !(args.output.is_dir()) {
        eprintln!(
            "Output path (-o) must be a directory: {}",
            args.output.display()
        );
        std::process::exit(1);
    }
    if !args.output.exists() {
        std::fs::create_dir_all(&args.output).expect("Failed to create output directory");
    }

    if let Err(e) = blackout_pdf(&args) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn blackout_pdf(args: &Args) -> anyhow::Result<()> {
    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            .or_else(|_| Pdfium::bind_to_system_library())?,
    );
    let document = pdfium.load_pdf_from_file(&args.input, args.password.as_deref())?;

    let render_config = PdfRenderConfig::new()
        .set_target_width(2480)
        .set_maximum_height(3508)
        .rotate_if_landscape(PdfPageRenderRotation::Degrees90, true);

    let mut pbar = pbar(Some(document.pages().len().into()));

    for (index, page) in document.pages().iter().enumerate() {
        let input_file_name = args
            .input
            .file_stem()
            .with_context(|| "input file name is invalid")?
            .to_str()
            .with_context(|| "input file name is invalid UTF-8")?;

        let output_path = args
            .output
            .join(format!("{}-page-{}.jpg", input_file_name, index));

        let image = page
            .render_with_config(&render_config)?
            .as_image()
            .into_rgb8();

        let image = image::DynamicImage::ImageRgb8(image).to_rgb8();
        let mut blackout_image = image::RgbImage::new(image.width(), image.height());

        for (x, y, pixel) in image.enumerate_pixels() {
            if pixel[0] == 255 && pixel[1] == 255 && pixel[2] == 255 {
                blackout_image.put_pixel(x, y, *pixel);
            } else {
                blackout_image.put_pixel(x, y, image::Rgb([0, 0, 0]));
            }
        }

        blackout_image
            .save_with_format(output_path, image::ImageFormat::Jpeg)
            .map_err(|_| PdfiumError::ImageError)?;

        pbar.update(1).unwrap();
    }

    pbar.close()
}
