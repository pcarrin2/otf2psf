use clap::Parser;
use std::fs;
use std::path::Path;
use crate::psf2_font::Psf2Font;
use ab_glyph::FontRef;

use log::{info};
use colog;
mod psf2_font;
mod glyph_image_owned;

#[derive(Parser, Debug)]
#[command(version, about)]
struct OtfToPsf2Options {
    /// A relative or absolute path to an OTF font file, which will be converted to a PSF2 font.
    otf_file: String,
    /// A path to an output file, where a generated PSF2 font will be stored.
    output_file: String,
    /// The target font height in pixels.
    #[arg(default_value_t = 16)]
    height: u32,
    /// A path to a file specifying a Unicode table.
    #[arg(short, long)]
    unicode_table_file: Option<String>,
}

fn main() { 
    colog::init();
    let cli_options = OtfToPsf2Options::parse();
    let otf_font_data = std::fs::read(&cli_options.otf_file).expect("missing input font file");
    let otf_font = FontRef::try_from_slice(&otf_font_data).expect("invalid input font file");
    let psf2font = Psf2Font::new(otf_font, cli_options.height, cli_options.unicode_table_file);
    let output_file = &Path::new(&cli_options.output_file);
    fs::write(output_file, psf2font.unwrap().write()).unwrap();
    info!("Wrote PSF2 font file.");
}
