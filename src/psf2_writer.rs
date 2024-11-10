use crate::glyph::Glyph;
use crate::ttf_parser::TtfParser;
use crate::errors::GlyphSetError;
use crate::unicode_table::UnicodeTable;


const PSF2_MAGIC_BYTES: [u8; 4] = [0x72, 0xb5, 0x4a, 0x86];
const PSF2_VERSION: [u8; 4] = [0x0, 0x0, 0x0, 0x0];
const PSF2_HEADER_SIZE: [u8; 4] = 32_u32.to_le_bytes();

/// Header information for a PSF2 font file.
pub struct Psf2Header {
    /// Specifies whether a Unicode mapping table is included for this font. If false, glyphs will
    /// represent `glyph_count` Unicode codepoints, starting from U+0, in order.
    pub unicode_table_exists: bool,
    /// The number of glyphs included in the font. Can be arbitrarily large, but the Linux kernel
    /// will only accept fonts up to 512 characters, I think.
    pub glyph_count: u32,
    /// The number of bytes used to store each glyph.
    pub glyph_size: u32,
    /// The height in pixels of each glyph.
    pub glyph_height: u32,
    /// The width in pixels of each glyph.
    pub glyph_width: u32,
}

impl Psf2Header {
    /// Writes the PSF2 header to an array of bytes.
    pub fn write(self) -> [u8; 32] {
        let flags: [u8;4] = (self.unicode_table_exists as u32).to_le_bytes();
        let mut header = [0u8; 32];
        header[ 0.. 4].clone_from_slice(&PSF2_MAGIC_BYTES);
        header[ 4.. 8].clone_from_slice(&PSF2_VERSION);
        header[ 8..12].clone_from_slice(&PSF2_HEADER_SIZE);
        header[12..16].clone_from_slice(&flags);
        header[16..20].clone_from_slice(&self.glyph_count.to_le_bytes());
        header[20..24].clone_from_slice(&self.glyph_size.to_le_bytes());
        header[24..28].clone_from_slice(&self.glyph_height.to_le_bytes());
        header[28..32].clone_from_slice(&self.glyph_width.to_le_bytes());
        return header
    }

}

/// A set of glyph bitmaps used in a PSF2 font file.
pub struct Psf2GlyphSet {
    /// A vector of glyph bitmaps. If a Unicode mapping table is present in the PSF2 font, these
    /// bitmaps correspond with table entries. If no Unicode mapping table is present, these
    /// bitmaps correspond with Unicode characters `U+0000` through `U+(glyph_count - 1)`.
    glyphs: Vec<Glyph>,
    /// The height of each glyph.
    pub height: u32,
    /// The width of each glyph.
    pub width: u32,
    /// The length of each glyph, in bytes.
    pub length: u32,
}

impl Psf2GlyphSet {
    pub fn new_with_unicode_table(ttf_parser: TtfParser, unicode_table: &UnicodeTable, pad: bool) 
        -> Result<Self, GlyphSetError> {
        let mut glyph_set: Vec<Glyph> = vec![];
        for equivalent_graphemes_list in unicode_table.data.iter() {
            // select a "reference grapheme" to rasterize and use as a symbol for a set of
            // equivalent graphemes.
            let reference_grapheme = &equivalent_graphemes_list[0];
            glyph_set.push(ttf_parser.render_string(reference_grapheme)?);
        }

        return match pad {
            true => Self::from_vec_of_glyphs_pad(glyph_set),
            false => Self::from_vec_of_glyphs_strict(glyph_set),
        }
        
    }

    pub fn new(ttf_parser: TtfParser, glyph_count: u32, pad: bool) -> Result<Self, GlyphSetError> {
        let glyph_set: Vec<Glyph> = (0..(glyph_count)).map(
            |i|
            ttf_parser.render_char(char::from_u32(i).expect("Invalid Unicode codepoint while generating glyph set"))
        ).collect();

        return match pad {
            true => Self::from_vec_of_glyphs_pad(glyph_set),
            false => Self::from_vec_of_glyphs_strict(glyph_set),
        }
    }

    fn from_vec_of_glyphs_pad(glyphs: Vec<Glyph>) -> Result<Self, GlyphSetError> {
        let mut max_height: u32 = 0;
        let mut max_width: u32 = 0;
        let mut max_length: u32 = 0;

        for g in glyphs.iter() {
            max_height = std::cmp::max(u32::from(g.height), max_height);
            max_width = std::cmp::max(u32::from(g.width), max_width);
            max_length = std::cmp::max(u32::try_from(g.data.len()).unwrap(), max_length);
        }

        let mut padded_glyphs: Vec<Glyph> = vec![];

        for g in glyphs.into_iter() {
            let padded = g.pad(max_height, max_width)?;
            padded_glyphs.push(padded);
        }


        return Self::from_vec_of_glyphs_strict(padded_glyphs);
    }

    fn from_vec_of_glyphs_strict(glyphs: Vec<Glyph>) -> Result<Self, GlyphSetError> {
        // check that all heights/widths/lengths are equal.
        let height: u32;
        let width: u32;
        let length: u32;

        let mut glyph_set_iter = glyphs.iter();
        let glyph_set_first = glyph_set_iter.nth(0);
        match glyph_set_first {
            None => return Ok(Self{
                glyphs, 
                height: 0, 
                width: 0, 
                length: 0,
            }),
            Some(f) => {
                (height, width, length) = (f.height, f.width, f.data.len() as u32);

                for g in glyph_set_iter {
                    if u32::from(g.height) != height 
                        || u32::from(g.width) != width {
                        return Err(GlyphSetError::InconsistentDimensions{
                            height: g.height, 
                            width: g.width, 
                            expected_height: height, 
                            expected_width: width,
                        })
                    }
                    else if g.data.len() != length as usize {
                        return Err(GlyphSetError::InconsistentLengths{length: g.data.len(), expected_length: length as usize});
                    }
                }

                return Ok(Self{glyphs, height, width, length});

            }
        }

    }

    pub fn write(self) -> Vec<u8> {
        return self.glyphs.into_iter().map(|g| g.data).flatten().collect();
    }
}

/// A PSF2 font.
pub struct Psf2Font {
    pub header: Psf2Header,
    pub glyphs: Psf2GlyphSet,
    pub unicode_table: Option<UnicodeTable>,
}

impl Psf2Font {
    pub fn write(self) -> Vec<u8> {
        let mut font: Vec<u8> = self.header.write().to_vec();
        eprintln!("Font header length: {}", font.len());
        let glyphs_data = self.glyphs.write();
        eprintln!("Glyph set length: {}", glyphs_data.len());
        //font.extend(self.glyphs.write());
        font.extend(glyphs_data);
        eprintln!("Total font length: {}", font.len());
        if let Some(uc) = self.unicode_table {
            font.extend(uc.write())
        };
        return font;
    }
}
