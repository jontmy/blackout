use anyhow::Context;
use clap::Parser;
use pdfium_render::prelude::*;
use std::path::PathBuf;
use tqdm::pbar;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_path: PathBuf,
    #[arg(short, long)]
    output_path: PathBuf,
    #[arg(short, long)]
    password: Option<String>,
}

fn main() {
    let args = Args::parse();

    if !(args.input_path.is_file()) {
        eprintln!(
            "Input path (-i) must be a file: {}",
            args.input_path.display()
        );
        std::process::exit(1);
    }
    if !args.input_path.exists() {
        eprintln!(
            "Input file (-i) does not exist: {}",
            args.input_path.display()
        );
        std::process::exit(1);
    }
    if let Some("pdf") = args.input_path.extension().and_then(|s| s.to_str()) {
    } else {
        eprintln!(
            "Input file (-i) must be a PDF file (extension .pdf): {}",
            args.input_path.display()
        );
        std::process::exit(1);
    }

    if args.output_path.extension().is_none() {
        std::fs::create_dir_all(&args.output_path).expect("Failed to create output directory");
    } else {
        // Create the parent directory for the output file, otherwise PdfDocument::save_to_file later will fail.
        std::fs::create_dir_all(
            args.output_path
                .parent()
                .expect("Failed to get parent directory"),
        )
        .expect("Failed to create output directory");
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
    let render_config = PdfRenderConfig::new()
        .set_target_width(2480)
        .set_maximum_height(3508)
        .rotate_if_landscape(PdfPageRenderRotation::Degrees90, true);

    let input_doc = pdfium.load_pdf_from_file(&args.input_path, args.password.as_deref())?;
    let mut output_doc = pdfium.create_new_pdf()?;

    let mut pbar = pbar(Some(input_doc.pages().len().into()));
    for input_page in input_doc.pages().iter() {
        let input_img = input_page
            .render_with_config(&render_config)?
            .as_image()
            .into_rgb8();

        let mut output_img = image::RgbImage::new(input_img.width(), input_img.height());
        for (x, y, pixel) in input_img.enumerate_pixels() {
            if pixel[0] == 255 && pixel[1] == 255 && pixel[2] == 255 {
                output_img.put_pixel(x, y, *pixel);
            } else {
                output_img.put_pixel(x, y, image::Rgb([0, 0, 0]));
            }
        }
        let output_img = image::DynamicImage::ImageRgb8(output_img);

        let mut output_page = output_doc
            .pages_mut()
            .create_page_at_end(PdfPagePaperSize::a4())?;

        let output_img_obj =
            PdfPageImageObject::new_with_width(&output_doc, &output_img, output_page.width())?;
        output_page.objects_mut().add_image_object(output_img_obj)?;

        pbar.update(1).unwrap();
    }

    let output_path = if args.output_path.extension().is_some() {
        args.output_path.clone()
    } else {
        let input_file_name = args
            .input_path
            .file_name()
            .with_context(|| "Input file name is invalid")?
            .to_str()
            .with_context(|| "Input file name is not valid UTF-8")?;
        args.output_path.join(input_file_name)
    };

    output_doc.save_to_file(&output_path)?;
    pbar.close()?;

    Ok(())
}
