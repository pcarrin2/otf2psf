use clap::Parser;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use ab_glyph::FontRef;

mod errors;
mod ttf_parser;
mod psf2_writer;
mod unicode_table;
mod glyph;

#[derive(Parser, Debug)]
#[command(version, about)]
struct OtfToPsf2Options {
    /// A relative or absolute path to an OTF font file, which will be converted to a PSF2 font.
    otf_file: PathBuf,
    /// A path to an output file, where a generated PSF2 font will be stored.
    output_file: PathBuf,
    /// The target font height in pixels.
    #[arg(default_value_t = 16)]
    height: u32,
    /// A path to a file specifying a Unicode table.
    #[arg(short, long)]
    unicode_table_file: Option<PathBuf>,
    /// The number of glyphs to include in the finished font. If a Unicode table is also specified,
    /// at most `glyph_count` glyphs will be included from the table. If a Unicode table is
    /// not specified, `glyph_count` glyphs will be generated, corresponding to Unicode codepoints
    /// `0` through `(glyph_count - 1)`.
    #[arg(short, long)]
    glyph_count: Option<u32>,
}

fn main() -> Result <(), Box<dyn std::error::Error>> { 
    let cli_options = OtfToPsf2Options::parse();

    let ttf_parser = ttf_parser::TtfParser::from_font_path(
        &cli_options.otf_file,
        cli_options.height,
    )?;

    let (unicode_table, glyph_count, glyphs) = match &cli_options.unicode_table_file {
        Some(p) => {
            let unicode_table = unicode_table::UnicodeTable::from_file(p, cli_options.glyph_count)?;
            let glyph_count = unicode_table.data.len() as u32;
            let glyphs = psf2_writer::Psf2GlyphSet::new_with_unicode_table(ttf_parser, &unicode_table)?;
            (Some(unicode_table), glyph_count, glyphs)
        }
        None => {
            let glyph_count = {if let Some(n) = cli_options.glyph_count {n} else {256}};
            (None, glyph_count, psf2_writer::Psf2GlyphSet::new(ttf_parser, glyph_count)?)
        }
    };

    let header = psf2_writer::Psf2Header{
        unicode_table_exists: cli_options.unicode_table_file.is_some(),
        glyph_count: glyph_count,
        glyph_size: glyphs.length,
        glyph_height: glyphs.height,
        glyph_width: glyphs.width,
    };

    let psf2font = psf2_writer::Psf2Font{
        header,
        glyphs,
        unicode_table,
    };
    let output_file = &Path::new(&cli_options.output_file);
    fs::write(output_file, psf2font.write())?;
    println!("Wrote PSF2 font file.");
    Ok(())
}
