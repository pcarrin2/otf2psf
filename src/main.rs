use clap::{Parser, Args, Subcommand};
use std::fs;
use std::path::Path;
use std::path::PathBuf;

mod errors;
mod ttf_parser;
mod psf2_writer;
mod unicode_table;
mod glyph;
mod report;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Report information about glyphs to stdout.
    Report(ReportOpts),
    /// Convert a TTF/OTF font to a PSF2 font.
    Convert(ConvertOpts),
}

#[derive(Debug, Args)]
#[clap(group = clap::ArgGroup::new("report-source").multiple(false))]
struct ReportOpts {
    /// A path to a TTF or OTF font file.
    ttf_file: PathBuf,
    /// The target font height, in pixels.
    #[clap(default_value_t = 16)]
    height: u32,
    /// Report on all the characters that would be used to generate a PSF2 font from 
    /// a Unicode mapping table.
    #[clap(long, group="report-source")]
    unicode_table_file: Option<PathBuf>,
    /// Report on a single character.
    #[clap(long, group="report-source")]
    single_character: Option<char>,
    /// Report on the Unicode block that contains a given character.
    #[clap(long, group="report-source")]
    block_containing: Option<char>,
}

#[derive(Debug, Args)]
struct ConvertOpts {
    /// A path to a TTF or OTF font file.
    ttf_file: PathBuf,
    /// A path to an output file, where a generated PSF2 font will be stored.
    output_file: PathBuf,
    /// The target font height, in pixels.
    #[clap(default_value_t = 16)]
    height: u32,
    /// A path to a file specifying a Unicode mapping table.
    #[clap(short, long)]
    unicode_table_file: Option<PathBuf>,
    /// The number of glyphs to include in the finished font. 
    // If a Unicode table is also specified, at most `glyph_count` glyphs will be included from the table. 
    // If a Unicode table is not specified, `glyph_count` glyphs will be generated, corresponding to 
    // Unicode codepoints `0` through `(glyph_count - 1)`. The default is the length of the Unicode table, 
    // if included, or 256 if no Unicode table is included.
    #[arg(short, long)]
    glyph_count: Option<u32>,
    /// Pad all glyphs to the canvas size of the largest glyph. 
    // Helpful for dealing with fonts where some special characters have unusually small canvases. 
    // If this flag is not set, this tool will require all glyphs to be the same size, and will exit 
    // with an error otherwise.
    #[arg(long, action)]
    pad: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_options = Cli::parse();
    return match cli_options.command {
        Command::Report(options) => {
            report(options)
        }
        Command::Convert(options) => {
            convert(options)
        }
    }
}

fn report(report_opts: ReportOpts) -> Result<(), Box <dyn std::error::Error>> {
    let ttf_file = &report_opts.ttf_file;
    let height = report_opts.height;
    let ttf_parser = ttf_parser::TtfParser::from_font_path(ttf_file, height)?;

    if let Some(uc) = &report_opts.unicode_table_file {
        crate::report::report_unicode_table(ttf_parser, uc)?;
    } else if let Some(block_char) = report_opts.block_containing {
        crate::report::report_unicode_block(ttf_parser, unicode_blocks::find_unicode_block(block_char)
            .ok_or("No Unicode block found matching character")?);
    } else if let Some(single_char) = report_opts.single_character {
        println!("{}", ttf_parser.report_char(single_char));
    }
    Ok(())
}

fn convert(convert_opts: ConvertOpts) -> Result <(), Box<dyn std::error::Error>> { 
    let ttf_file = &convert_opts.ttf_file;
    let height = convert_opts.height;
    let unicode_table_file = &convert_opts.unicode_table_file;
    let output_file = &convert_opts.output_file;
    let cli_glyph_count = convert_opts.glyph_count;
    let pad = convert_opts.pad;


    let ttf_parser = ttf_parser::TtfParser::from_font_path(
        ttf_file,
        height,
    )?;

    let (unicode_table, glyph_count, glyphs) = match unicode_table_file {
        Some(p) => {
            let unicode_table = unicode_table::UnicodeTable::from_file(p, cli_glyph_count)?;
            let uc_table_glyph_count = unicode_table.data.len() as u32;
            let glyphs = psf2_writer::Psf2GlyphSet::new_with_unicode_table(ttf_parser, &unicode_table, pad)?;
            (Some(unicode_table), uc_table_glyph_count, glyphs)
        }
        None => {
            let glyph_count = {if let Some(n) = cli_glyph_count {n} else {256}};
            (None, glyph_count, psf2_writer::Psf2GlyphSet::new(ttf_parser, glyph_count, pad)?)
        }
    };

    eprintln!("Glyph count: {}", glyph_count);

    let header = psf2_writer::Psf2Header{
        unicode_table_exists: unicode_table_file.is_some(),
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
    let output_file = &Path::new(output_file);
    fs::write(output_file, psf2font.write())?;
    println!("Wrote PSF2 font file.");
    Ok(())
}
