use crate::glyph::Glyph;
use crate::errors::GlyphSetError;


const PSF2_MAGIC_BYTES: [u8; 4] = [0x72, 0xb5, 0x4a, 0x86];
const PSF2_VERSION: [u8; 4] = [0x0, 0x0, 0x0, 0x0];
const PSF2_HEADER_SIZE: [u8; 4] = 32_u32.to_le_bytes();

/// Header information for a PSF2 font file.
pub struct Psf2Header {
    /// Specifies whether a Unicode mapping table is included for this font. If false, glyphs will
    /// represent `glyph_count` Unicode codepoints, starting from U+0, in order.
    unicode_table_exists: bool,
    /// The number of glyphs included in the font. Can be arbitrarily large, but the Linux kernel
    /// will only accept fonts up to 512 characters, I think.
    glyph_count: u32,
    /// The number of bytes used to store each glyph.
    glyph_size: u32,
    /// The height in pixels of each glyph.
    glyph_height: u32,
    /// The width in pixels of each glyph.
    glyph_width: u32,
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
    height: u32,
    /// The width of each glyph.
    width: u32,
    /// The length of each glyph, in bytes.
    length: u32,
}

impl Psf2GlyphSet {
    pub fn new_with_unicode_table(ttf_parser: TtfParser, unicode_table: &unicode_table::UnicodeTable) 
        -> Result<Self, GlyphError> {
        let mut glyph_set: Vec<Psf2Glyph> = vec![];
        for equivalent_graphemes_list in unicode_table.data.iter() {
            // select a "reference grapheme" to rasterize and use as a symbol for a set of
            // equivalent graphemes.
            let reference_grapheme = &equivalent_graphemes_list[0];
            glyph_set.push(ttf_parser.render_string(reference_grapheme));
        }

        return Self::from_vec_of_glyphs(glyph_set);
        
    }

    pub fn new(ttf_parser: TtfParser, glyph_count: u32) -> Self {
        let mut glyph_set: Vec<Psf2Glyph> = vec![];
        let mut i = 0;
        let mut c = char::from(0);
        while i < glyph_count {
            glyph_set.push(ttf_parser.render_string(c));
            c = c.forward(1);
            i++;
        }

        return Self::from_vec_of_glyphs(glyph_set);
    }

    fn from_vec_of_glyphs(glyphs: Vec<Glyph>) -> Result<Self, GlyphSetError> {
        // check that all heights/widths/lengths are equal.
        let height: u32;
        let width: u32;
        let length: u32;

        let glyph_set_iter = glyphs.iter();
        let glyph_set_first = glyph_set_iter.nth(0);
        (height, width, length) = (glyph_set_first.height, glyph_set_first.width, glyph_set_first.length);

        for g in glyph_set_iter {
            if (g.height != height || g.width != width) {
                return GlyphSetError::InconsistentDimensions{g.height, g.width, height, width}
            }
            else if (g.length != length) {
                return GlyphSetError::InconsistentLengths{g.length, length}
            }
        }

        return Ok(Self{glyphs, height, width, length});

    }

    pub fn write(self) -> Vec<u8> {
        return self.glyphs.into_iter().map(|g| g.data).flatten().collect();
    }
}

/// A PSF2 font.
pub struct Psf2Font {
    header: Psf2Header,
    glyphs: Psf2GlyphSet,
    unicode_table: Option<Psf2UnicodeTable>,
}

impl Psf2Font {
    pub fn write(self) -> Vec<u8> {
        let mut font: Vec<u8> = self.header.write().to_vec();
        font.extend(self.glyphs.write());
        font.extend(self.unicode_table.write());
        return font;
    }
}
