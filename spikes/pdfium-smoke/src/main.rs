use std::{env, error::Error, path::PathBuf};

use pdfium_render::prelude::{PdfRenderConfig, Pdfium};

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = env::args_os().skip(1);
    let library_directory = PathBuf::from(arguments.next().ok_or("missing library directory")?);
    let input_pdf = PathBuf::from(arguments.next().ok_or("missing input PDF")?);
    let output_png = PathBuf::from(arguments.next().ok_or("missing output PNG")?);

    let bindings = Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(
        &library_directory,
    ))?;
    let pdfium = Pdfium::new(bindings);
    let document = pdfium.load_pdf_from_file(&input_pdf, None)?;
    let page = document.pages().get(0)?;
    let image = page
        .render_with_config(
            &PdfRenderConfig::new()
                .set_target_width(1200)
                .render_form_data(true),
        )?
        .as_image()?;

    image.save(output_png)?;
    Ok(())
}
