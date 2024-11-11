use crate::errors::GlyphError;
use ab_glyph::v2::GlyphImage;
use ab_glyph::GlyphImageFormat;
use bitvec::prelude::*;


/// A glyph bitmap, in psf2 style: mono-color, one bit per pixel, byte-padded rows.
pub struct Glyph {
    pub height: u32,
    pub width: u32,
    pub data: Vec<u8>,
    pub grapheme: String,
}

impl Glyph {
    /// Combines `self` and `other`'s bitmaps with a logical OR, and appends `other`'s grapheme to
    /// `self`'s, in a new returned `Glyph` struct. Intended for adding combining diacritics.
    /// Returns an error if the heights, widths, or lengths of `self` and `other` do not match.
    pub fn add(self, other: Self) -> Result<Self, GlyphError> {
        if self.height != other.height || self.width != other.width {
            return Err(GlyphError::WrongDimensions{
                    height: self.height, 
                    width: self.width, 
                    expected_height: other.height, 
                    expected_width: other.width,
                    }
                );
        }
        if self.data.len() != other.data.len() {
            return Err(GlyphError::WrongLength{length: self.data.len(), expected_length: other.data.len()});
        }

        let mut grapheme = self.grapheme;
        grapheme.push_str(&other.grapheme);

        // bitwise OR the bytes of self's and other's data: this "overlays" the bitmaps on top of
        // each other.
        let data = self.data.into_iter().zip(other.data.into_iter())
            .map( |(a,b)| a | b )
            .collect::<Vec<_>>();

        let height = self.height;
        let width = self.width;

        return Ok(Self{height, width, data, grapheme})
    }

    /// Pads `self` to given dimensions `new_height` and `new_width`. Inserts blank space to the
    /// right of `self` and below it. Returns an error if the padded dimensions are too small to
    /// fit `self`.
    pub fn pad(self, new_height: u32, new_width: u32) -> Result<Self, GlyphError> {
        if self.height > new_height || self.width > new_width {
            return Err(GlyphError::PadTooSmall{height: self.height, width: self.width, pad_height: new_height, pad_width: new_width});
        }
        let mut data: Vec<u8> = vec![];
        let self_row_length = (self.width as f64 / 8.0).ceil() as usize;
        let padded_row_length = (new_width as f64 / 8.0).ceil() as usize;
        if self_row_length < padded_row_length {
            for chunk in self.data.chunks_exact(self_row_length.into()) {
                data.append(&mut (chunk.to_owned().to_vec()));
                data.append(&mut vec![0u8; padded_row_length - self_row_length]);
            }
        } else {
            data = self.data;
        }
        data.append(&mut vec![0u8; (new_height - self.height) as usize * padded_row_length]);
        return Ok(Self{height: new_height, width: new_width, data, grapheme: self.grapheme});
    }

    /// Creates a new `Glyph` from an embedded bitmap in a TTF/OTF file.
    pub fn from_glyph_image(glyph_image: GlyphImage, grapheme: char) -> Result<Self, GlyphError> {
        return match glyph_image.format {
            GlyphImageFormat::BitmapMono => {
                Ok(Glyph {
                    height: glyph_image.height as u32,
                    width: glyph_image.width as u32,
                    data: glyph_image.data.to_vec(),
                    grapheme: grapheme.to_string(),
                })
            }

            GlyphImageFormat::BitmapMonoPacked => {
                let mut data = bitvec![u8, Msb0; 0; 0];
                let whitespace_width = ((glyph_image.width as f64 / 8.0).ceil() as usize) * 8 - glyph_image.width as usize;
                let whitespace = bitvec![u8, Msb0; 0; whitespace_width as usize];
                let mut glyph_image_clone = glyph_image.data.to_vec();

                let glyph_image_rows = glyph_image_clone.view_bits_mut::<Msb0>().chunks_exact_mut(glyph_image.width.into());
                for row in glyph_image_rows {
                    let mut padded_row = row.to_bitvec();
                    padded_row.append(&mut whitespace.clone());
                    data.extend(padded_row);
                }

                let data_vec = data.into_vec();
                
                Ok(Glyph {
                    height: glyph_image.height as u32,
                    width: glyph_image.width as u32,
                    data: data_vec,
                    grapheme: grapheme.to_string(),
                })
            }
            _fmt => Err(GlyphError::GlyphImgFmtUnsupported{format: _fmt}),
        }
    }
}
