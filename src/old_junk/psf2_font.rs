use std::error;
use std::io::BufRead;
use std::iter::once;
use itertools::Itertools;
use log::{warn, info, trace, Level};
use unicode_segmentation::UnicodeSegmentation;
use regex::Regex;
use ab_glyph::{FontRef, PxScaleFont};
use crate::glyph_image_owned;
use crate::grapheme_info;

const PSF2_MAGIC_BYTES: [u8; 4] = [0x72, 0xb5, 0x4a, 0x86];
const PSF2_VERSION: [u8; 4] = [0x0, 0x0, 0x0, 0x0];
const PSF2_HEADER_SIZE: [u8; 4] = 32_u32.to_le_bytes();

/// Header information for a PSF2 font file.
struct Psf2Header {
    /// Specifies whether a custom Unicode table is included for this font. If false, glyphs will
    /// represent `glyph_count` Unicode codepoints, starting from U+0, in order.
    write_unicode_table: bool,
    /// The number of glyphs included in the font. Can be arbitrarily large (up to the maximum size
    /// of a u32).
    glyph_count: u32,
    /// The number of bytes used to store each glyph.
    glyph_size: u32,
    /// The height in pixels of each glyph.
    glyph_height: u32,
    /// The width in pixels of each glyph.
    glyph_width: u32,
}

impl Psf2Header {
    /// Writes the PSF2 header to a vector of bytes.
    fn write(self) -> Vec<u8> {
        trace!("Writing the header to a vector of bytes.");
        let flags: Vec<u8> =  if self.write_unicode_table {
            1_u32.to_le_bytes().to_vec()
        } else {
            0_u32.to_le_bytes().to_vec()
        };
        trace!("Header flags: {flags:?}");
        let mut header: Vec<u8> = vec![];
        header.extend(PSF2_MAGIC_BYTES.to_vec());
        header.extend(PSF2_VERSION.to_vec());
        header.extend(PSF2_HEADER_SIZE.to_vec());
        header.extend(flags);
        header.extend(self.glyph_count.to_le_bytes());
        header.extend(self.glyph_size.to_le_bytes());
        header.extend(self.glyph_height.to_le_bytes());
        header.extend(self.glyph_width.to_le_bytes());
        return header
    }

    /// Convenience function to generate a PSF2 header.
    fn new(write_unicode_table: bool, glyph_count: u32, glyph_size: u32, glyph_height: u32, glyph_width: u32)
        -> Psf2Header 
    {
        trace!("Creating a new header.");
        return Psf2Header { write_unicode_table, glyph_count, glyph_size, glyph_height, glyph_width }
    }
}

/// A set of glyph bitmaps used in a PSF2 file.
#[derive(Debug)]
struct Psf2GlyphSet {
    /// A vector of glyph bitmaps.
    glyphs: Vec<Psf2Glyph>,
    /// The height of each glyph. (All glyph heights should be the same.)
    height: u32,
    /// The width of each glyph. (All glyph widths should be the same.)
    width: u32,
    /// The length of each glyph, in bytes, with padding. (All glyph lengths should be the same.)
    length: u32
}

impl Psf2GlyphSet {
    /// Writes a glyph set to a vector of bytes.
    pub fn write(self) -> Vec<u8> {
        trace!("Writing the glyph set to a vector of bytes.");
        return self.glyphs
            .into_iter()
            .map(|g| g.write())
            .flatten()
            .collect::<Vec<_>>();
    }

    /// Creates a new glyph set from a TTF/OTF font, a target font height, and a Unicode table.
    pub fn new(font: FontRef, height: u32, unicode_table: &Psf2UnicodeTable)
              -> Result<Psf2GlyphSet, Box<dyn error::Error>> 
    {
        trace!("Creating a glyph set from a TTF or OTF font.");
        let mut glyph_set: Vec<Psf2Glyph> = vec![];
        let scaled_font = PxScaleFont{ font, scale: (height as f32).into()};
        for equivalent_graphemes_list in unicode_table.data.iter() {
            // select a "reference grapheme" to rasterize and use as a symbol for a set of 
            // equivalent graphemes.
            let reference_grapheme = &equivalent_graphemes_list[0];
            let glyph = Psf2Glyph::new(reference_grapheme, &scaled_font)?;
            glyph_set.push(glyph);
        }

        let heights_widths_lengths: Vec<_> = 
            glyph_set.iter()
            .map( |g| (g.height, g.width, g.length) )
            .unique().collect();
        let [(height, width, length)] = heights_widths_lengths.as_slice() else {
            error!("Character heights/widths/lengths differ, falling into these buckets: {heights_widths_lengths:?}");
            // println!("{glyph_set:?}");
            return Err("Different glyphs in the generated glyph set have different dimensions. Make sure this is a monospace font.".into());
        }; // TODO make this an error for real
        Ok(Psf2GlyphSet{glyphs: glyph_set, height: *height, width: *width, length: *length})
    }
}

/// A glyph bitmap used in a PSF2 file.
#[derive(Debug)]
struct Psf2Glyph {
    /// The bitmap itself, as a vector of bytes.
    bitmap: Vec<u8>, 
    /// The grapheme that was rendered to generate the glyph.
    grapheme: String,
    /// The height of the bitmap in pixels.
    height: u32,
    /// The width of the bitmap in pixels.
    width: u32,
    /// The length of the bitmap in bytes.
    length: u32
}

impl Psf2Glyph {
    /// Writes a glyph to a vector of bytes.
    pub fn write(self) -> Vec<u8> {
        trace!("Writing the bitmap for '{}' () to bytes.", self.grapheme);
        return self.bitmap;
    }
    
    /// Creates a new glyph bitmap, given a single Unicode grapheme, an ab_glyph font, 
    /// and a target font height in pixels.
    pub fn new(grapheme: &str, font: &PxScaleFont<FontRef>) -> Result<Psf2Glyph, Box<dyn error::Error>> {
        assert!(grapheme.len() > 0);

        let gr_info = grapheme_info::seq_info(grapheme);
        trace!("Creating bitmap for '{grapheme}' ({gr_info}).");

        let graphemes_count = UnicodeSegmentation::graphemes(grapheme, true).count();
        if graphemes_count != 1 {
            warn!("The Unicode sequence {grapheme} in the Unicode table encodes zero or multiple graphemes.");
        }

        let mut glyph_images = grapheme
            .chars()
            .map( |c| glyph_image_owned::GlyphImageOwned::new(c, &font)
                .convert_format(&ab_glyph::GlyphImageFormat::BitmapMono)?
                );

        let mut canvas = glyph_images.nth(0).unwrap();

        for img in glyph_images {
            canvas = canvas.overlay(img)?;
        }

        canvas.clone().draw_to_ascii_art(Level::Trace)?; //TODO: this sucks make it suck less

        let width = canvas.width;
        let height = canvas.height;
        let length = canvas.data.len();
        let bitmap = canvas.data;

        let glyph = Psf2Glyph {
            bitmap,
            grapheme: grapheme.to_string(),
            height: height.into(),
            width: width.into(),
            length: length.try_into()?
        };

        return Ok(glyph);
    }
}

/// A table specifying which Unicode graphemes map to each glyph in a PSF2 font.
struct Psf2UnicodeTable {
    data: Vec<Vec<String>>,
}

impl Psf2UnicodeTable {
    /// Generates a new Unicode table from a charset file.
    ///
    /// Each line of the charset file lists a series of Unicode codepoints/sequences that all map
    /// to one glyph in the final PSF2 font. Equivalent codepoints/sequences are comma-separated.
    ///
    /// For example, the line `U+xxxx, U+yyyy U+zzzz` indicates that one glyph should represent
    /// the single Unicode character U+xxxx and the sequence U+yyyy U+zzzz.
    ///
    /// Comments starting with `#` and empty lines are ignored.
    pub fn new(charset_path: &str) -> Result<Psf2UnicodeTable, Box<dyn error::Error>> { // set max char limit
        trace!("Creating a new Unicode table based on '{charset_path}'.");
        let mut table: Vec<Vec<String>> = vec![];
        let workdir = std::env::current_dir()?;
        let charset_path = workdir.join(charset_path);
        let charset_data = std::fs::File::open(&charset_path)?;
        let charset_lines = std::io::BufReader::new(charset_data).lines();
        trace!("Successfully loaded Unicode table file.");

        for line in charset_lines.flatten() {
            // a line could look like: "U+xxxx, U+yyyy U+zzzz"
            // this would mean U+xxxx and the sequence U+yyyy U+zzzz should be treated as equivalent
            let line_without_comments = line.split('#').nth(0)
                .ok_or::<String>("Error splitting line in character list".into())?.to_uppercase();
            let graphemes = line_without_comments
                .split(',').map(|x| x.trim());
            let mut table_entry: Vec<String> = vec![];
            let unicode_regex = Regex::new(r"(U\+):([0-9]+)")?;
            for g in graphemes {
                let mut chars = "".to_string();
                for (_, [_, charcode]) in unicode_regex.captures_iter(g).map(|c| c.extract()) {
                    let character_from_code_raw = char::from_u32(u32::from_str_radix(charcode, 16)?);
                    let Some(character_from_code) = character_from_code_raw else {
                        return Err("Invalid Unicode codepoint in character list".into());
                    };
                    chars.push(character_from_code);
                }
                if chars.len() != 0 {
                    table_entry.push(chars);
                }
            }
            if table_entry.len() != 0 {
                table.push(table_entry);
            }
        }

        Ok(Psf2UnicodeTable{data: table})

    }

    /// Generates a new minimal Unicode table for internal use when generating PSF2 font files
    /// without a user-specified Unicode table. This table will not be written to the final font
    /// file. It generates a table of the first `num_chars` code points, with no equivalencies.
    pub fn new_minimal_table(num_chars: u32) -> Result<Psf2UnicodeTable, Box<dyn error::Error>> {
        trace!("Creating a new minimal Unicode table with {num_chars:?} characters.");
        let mut data: Vec<Vec<String>> = vec![];
        for i in 0..num_chars {
            // TODO: iterate over valid Unicode codepoints instead of all integers
            // won't matter for normal-sized PSF files (512 chars or so), but will matter for large fonts.
            let unicode_character = char::from_u32(i)
                .ok_or::<String>("Invalid Unicode codepoint during minimal Unicode table generation".into())?;
            let table_entry: Vec<String> = vec![unicode_character.to_string()];
            data.push(table_entry);
        }
        return Ok(Psf2UnicodeTable{data});
    }

    /// Writes a PSF2 Unicode table to a vector of bytes.
    pub fn write(self) -> Vec<u8> {
        trace!("Writing the Unicode table to a vector of bytes.");
        let ss: u8 = 0xfe;
        let term: u8 = 0xff;

        // TODO: clean up this section.

        let mut unicode_table: Vec<u8> = vec![];
        for equivalent_graphemes_list in self.data.into_iter() {
            // partition into single-character and multi-character graphemes
            let (sc, mc): (Vec<String>, Vec<String>) = equivalent_graphemes_list
                .into_iter().partition(|grapheme| grapheme.len() == 1);
            let row: Vec<u8> = sc.into_iter().map(|grapheme| grapheme.as_bytes().to_vec())
                                   .chain(mc.into_iter().map(|grapheme| { 
                                       let mut grapheme_bytes = vec![ss]; 
                                       grapheme_bytes.extend(grapheme.as_bytes()); 
                                       grapheme_bytes 
                                   } ))
                                   .chain(once(vec![term]))
                                   .flatten()
                                   .collect();
            unicode_table.extend(row);
        }

        return unicode_table;
    }
}

/// A PSF2 font, which can be generated and written to disk. No support is provided (yet) for
/// reading PSF2 fonts into this struct.
pub struct Psf2Font {
    /// A header for the PSF2 font.
    header: Psf2Header, 
    /// A glyph set for the PSF2 font.
    glyphs: Psf2GlyphSet,
    /// A Unicode table for the PSF2 font.
    unicode_table: Psf2UnicodeTable,
    /// Whether to write the Unicode table to disk -- if a minimal Unicode table is being used,
    /// this field should be set to `false`. If a user-specified Unicode table is being used, this
    /// field should be set to `true`.
    write_unicode_table: bool
}

impl Psf2Font {
    /// Generates a PSF2 font from a rusttype Font. Since PSF2 fonts are black-and-white bitmaps,
    /// there's no possibility for anti-aliasing -- vector fonts will look quite ugly, and it's
    /// better to use fonts that have bitmaps available at certain sizes, like Terminus or Unifont.
    pub fn new(input_font: FontRef, 
               height: u32, 
               unicode_table_file: Option<String>
               ) -> Result<Psf2Font, Box<dyn error::Error>> {
        info!("Initializing a new PSF2 font.");
        let (unicode_table, write_unicode_table) = if let Some(unicode_table_file) = &unicode_table_file {
            (Psf2UnicodeTable::new(unicode_table_file)?, true)
        } else {
            (Psf2UnicodeTable::new_minimal_table(512)?, false) // make user-defined?
        };

        let glyph_count: u32 = unicode_table.data.len().try_into()?;
        info!("This font will contain {glyph_count} glyphs.");
        let glyphs = Psf2GlyphSet::new(input_font, height, &unicode_table)?;
        let header = Psf2Header::new(
            write_unicode_table,
            glyph_count,
            glyphs.length,
            glyphs.height,
            glyphs.width); 
        Ok(Psf2Font{header, glyphs, unicode_table, write_unicode_table})
    }

    /// Writes a PSF2 font to a vector of bytes, for writing to disk.
    pub fn write(self) -> Vec<u8> {
        info!("Writing the PSF2 font to a vector of bytes.");
        let mut data: Vec<u8> = vec![];
        data.append(&mut self.header.write());
        data.append(&mut self.glyphs.write());
        if self.write_unicode_table {
            data.append(&mut self.unicode_table.write());
        }
        return data;
    }
}
