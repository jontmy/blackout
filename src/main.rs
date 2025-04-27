use anyhow::Context;
use clap::Parser;
use itertools::Itertools;
use pdfium_render::prelude::*;
use std::path::PathBuf;
use tqdm::Iter;

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

    if args.input_path.is_file() {
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
    }

    // Ensure output path is a directory if input path is a directory
    if !args.output_path.is_dir() && args.input_path.is_file() {
        eprintln!("Output path (-o) must be a directory since input path (-i) is a directory");
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

    // Find all input PDF files
    let input_paths = if args.input_path.extension().is_some() {
        vec![args.input_path.clone()]
    } else {
        std::fs::read_dir(args.input_path)
            .expect("Failed to read input directory")
            .into_iter()
            .flat_map(|f| f)
            .map(|f| f.path())
            .filter(|p| {
                p.is_file()
                    && p.extension()
                        .is_some_and(|ext| ext.to_ascii_lowercase() == "pdf")
            })
            .collect_vec()
    };

    let err_count = input_paths
        .into_iter()
        .map(|input_path| blackout_pdf(&input_path, &args.output_path, args.password.as_deref()))
        .filter_map(Result::err)
        .inspect(|err| eprintln!("{err}"))
        .count();

    if err_count > 0 {
        std::process::exit(1);
    }
}

fn blackout_pdf(
    input_path: &PathBuf,
    output_path: &PathBuf,
    password: Option<&str>,
) -> anyhow::Result<()> {
    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            .or_else(|_| {
                Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("../"))
            })
            .or_else(|_| {
                Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("../../"))
            })
            .or_else(|_| Pdfium::bind_to_system_library())?,
    );
    let render_config = PdfRenderConfig::new()
        .set_target_width(2480)
        .set_maximum_height(3508)
        .rotate_if_landscape(PdfPageRenderRotation::Degrees90, true);

    let input_doc = pdfium.load_pdf_from_file(input_path, password)?;
    let mut output_doc = pdfium.create_new_pdf()?;

    let page_count = input_doc.pages().len().into();
    for input_page in input_doc
        .pages()
        .iter()
        .take(page_count)
        .tqdm()
        .desc(Some(input_path.display()))
    {
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
    }

    let output_path = if output_path.extension().is_some() {
        output_path.clone()
    } else {
        let input_file_name = input_path
            .file_name()
            .with_context(|| "Input file name is invalid")?
            .to_str()
            .with_context(|| "Input file name is not valid UTF-8")?;
        output_path.join(input_file_name)
    };

    output_doc.save_to_file(&output_path)?;
    Ok(())
}
