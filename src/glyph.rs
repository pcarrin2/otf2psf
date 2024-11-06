use crate::errors::GlyphError;
use ab_glyph::v2::GlyphImage;
use ab_glyph::GlyphImageFormat;

/// A glyph bitmap, in psf2 style: mono-color, one bit per pixel, byte-padded rows.
pub struct Glyph {
    height: u8,
    width: u8,
    data: Vec<u8>,
    grapheme: String,
}

impl Glyph {
    /// Combines `self` and `other`'s bitmaps with a logical OR, and appends `other`'s grapheme to
    /// `self`'s, in a new returned `Glyph` struct. Intended for adding combining diacritics.
    /// Returns an error if the heights, widths, or lengths of `self` and `other` do not match.
    pub fn add(self, other: Self) -> Result(Self, GlyphError) {
        if self.height != other.height || self.width != other.width {
            return GlyphError::WrongDimensions(self.height, self.width, other.height, other.width)
        }
        if self.data.length() != other.data.length() {
            return GlyphError::WrongLength(self.data.length(), other.data.length())
        }

        let mut sum = self;
        sum.grapheme = self.grapheme.push_str(&other.grapheme);

        // bitwise OR the bytes of self's and other's data: this "overlays" the bitmaps on top of
        // each other.
        sum.data = self.data.into_iter().zip(other.data.into_iter())
            .map( |(a,b)| a | b )
            .collect::Vec<_>();

        return Ok(sum)
    }

    /// Pads `self` to given dimensions `new_height` and `new_width`. Inserts blank space to the
    /// right of `self` and below it. Returns an error if the padded dimensions are too small to
    /// fit `self`.
    pub fn pad(self, new_height: u8, new_width: u8) -> Result(Self, GlyphError) {
        if self.height > new_height || self.width > new_width {
            return GlyphError::PadTooSmall{self.height, self.width, new_height, new_width}
        }
        let mut padded = self;
        padded.height = new_height;
        padded.width = new_width;
        let self_row_length = (self.width / 8.0).ceil() as u8;
        let padded_row_length = (padded.width / 8.0).ceil() as u8;
        if self_row_length < padded_row_length {
            padded.data: Vec<u8> = self.data.into_iter()
                .chunks_exact(self_row_length)
                .for_each(append(vec![0u8, padded_row_length - self_row_length]))
                .flatten()
                .collect();
        }
        padded.data.append(vec![0u8, padded_row_length * (padded.height - self.height)]);
        return Ok(padded)
    }

    /// Creates a new `Glyph` from an embedded bitmap in a TTF/OTF file.
    // TODO add BitmapMonoPacked support!
    pub fn from_glyph_image(glyph_image: GlyphImage, grapheme: char) -> Result(Self, GlyphError) {
        return match glyph_image.format {
            GlyphImageFormat::BitmapMono => {
                Glyph {
                    height = glyph_image.height,
                    width = glyph_image.width,
                    data = glyph_image.data,
                    grapheme = grapheme.to_string()
                }
            }
            _fmt => GlyphError::GlyphImgFmtUnsupported{_fmt},
        }
    }
}
