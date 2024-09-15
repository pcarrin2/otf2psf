use std::error;
use ab_glyph::{Point, FontRef, Font, ScaleFont, Glyph, GlyphImageFormat, GlyphImageFormat::*};
use ab_glyph::point;
use ab_glyph::PxScaleFont;
use ab_glyph::v2::GlyphImage;
use itertools::Itertools;
use bitvec::prelude::*;
use std::iter::once;
use log::{error, warn, info, debug, trace};

/// A struct similar to `ab_glyph::v2::GlyphImage`, but the `data` field is owned instead of sliced
/// from a font file, and the struct is not labeled non-exhaustive. This means that
/// `GlyphImageOwned` structs can be converted between glyph image formats more easily. Vector
/// glyphs can also be rasterized into this format.
#[derive(Debug, Clone)]
pub struct GlyphImageOwned {
    pub origin: Point,
    pub width: u16,
    pub height: u16,
    pub pixels_per_em: u16,
    pub data: Vec<u8>,
    pub format: GlyphImageFormat,
}

impl GlyphImageOwned {
    pub fn new(character: char, font: &PxScaleFont<FontRef>) -> Self {
        trace!("Creating new GlyphImageOwned bitmap for '{:?}'", character);
        let glyph_img_option = Self::try_find_and_convert(character, font);
        match glyph_img_option {
            Some(glyph_img) => { return glyph_img }
            None => {
                warn!("No embedded bitmap for '{:?}', rasterizing a vector graphic instead.", character);
                let rasterized_glyph_img = Self::rasterize(character, font);
                return rasterized_glyph_img;
            }
        }
    }

    pub fn try_find_and_convert(character: char, font: &PxScaleFont<FontRef>) 
        -> Option<Self> 
    {
        let glyph_id = font.glyph_id(character);
        let glyph = font.font.glyph_raster_image2(glyph_id, font.height().ceil() as u16)?;
        let glyph: GlyphImageOwned = glyph.into();
        return Some(glyph);
    }

    pub fn rasterize(character: char, font: &PxScaleFont<FontRef>) -> Self {
        let glyph: Glyph = font
            .glyph_id(character)
            .with_scale_and_position(font.height(), point(0.0, 0.0));
        let width = font.h_advance(glyph.id).ceil() as u16;
        let height = font.height() as u16;

        debug!("Rasterizing character '{character:?}' onto canvas with height {height} and width {width}.");
        let mut data = bitvec![u8, Msb0; 0; (width * height).into()];
        if let Some(og) = font.outline_glyph(glyph) {
            let bounds = og.px_bounds();
            og.draw( |x, y, v| {
                let x = (x as f32 + bounds.min.x) as u16;
                let y = (y as f32 + bounds.min.y) as u16;

                if x < width && y < height && v >= 0.5 {
                    data.set((x as usize) + (y as usize) * (width as usize), true);
                }
            })
        }

        let data = data.into_vec();
        let origin = point(0.0, 0.0);
        let pixels_per_em = height;
        let format = BitmapMonoPacked;

        return GlyphImageOwned{ origin, width, height, pixels_per_em, data, format };
    }

    /// Converts the data in a GlyphImageOwned into a given GlyphImageFormat. Currently only
    /// implemented for a couple possible formats.
    pub fn convert_format(self, fmt: &GlyphImageFormat) -> Result<Self, Box<dyn error::Error>> {
        let mut converted: GlyphImageOwned = self.clone();
        converted.format = fmt.clone();
        trace!("Converting a {:?} glyph to the {:?} format.", self.format, fmt);
        match (self.format, fmt) {
            (BitmapMonoPacked, BitmapMono) => {
                if self.width % 8 != 0 {
                    trace!("Glyph width isn't a multiple of 8; padding each line.");
                    let padded_width = (self.width as f32 / 8.0).ceil() as u16 * 8;
                    let unpadded = self.data.view_bits::<Msb0>();
                    let padding = bitvec![u8, Msb0; 0; (padded_width - self.width).into()];
                    let padded: BitVec<u8, Msb0> = unpadded
                        .chunks(self.width.into())
                        .intersperse(&padding.as_bitslice())
                        .chain(once(padding.as_bitslice()))
                        .flatten()
                        .collect();

                    converted.data = padded.into_vec();
                }
            }

            (BitmapMono, BitmapMonoPacked) => {
                if self.width % 8 != 0 {
                    trace!("Glyph width isn't a multiple of 8; packing each line.");
                    let padded_width = (self.width as f32/8.0).ceil() as u16 * 8;
                    let padded = self.data.view_bits::<Msb0>();
                    let width = self.width as usize;
                    let packed: BitVec<u8, Msb0> = padded
                        .chunks(padded_width.into())
                        .map( |x| &x[0..width] )
                        .flatten()
                        .collect();

                    converted.data = packed.into_vec();
                }
            }

            (BitmapMono, BitmapMono) => (),
            (BitmapMonoPacked, BitmapMonoPacked) => (),
            (Png, Png) => (),
            (BitmapGray2, BitmapGray2) => (),
            (BitmapGray2Packed, BitmapGray2Packed) => (),
            (BitmapGray4, BitmapGray4) => (),
            (BitmapGray4Packed, BitmapGray4Packed) => (),
            (BitmapGray8, BitmapGray8) => (),
            (BitmapPremulBgra32, BitmapPremulBgra32) => (),

            _ => {return Err("Unsupported embedded bitmap format.".into());}
        }
        return Ok(converted);
    }

    /// Overlays the bitmaps of two `OwnedGlyphImage`s. Useful for composing glyphs from Unicode
    /// sequences that involve diacritics.
    pub fn overlay(self, other: Self) -> Result<Self, Box<dyn error::Error>> {
        let other = other.convert_format(&self.format)?;

        if self.width != other.width 
            || self.height != other.height
            || self.pixels_per_em != other.pixels_per_em
            || self.origin != other.origin {
            return Err("Trying to overlay OwnedGlyphImage bitmaps, but their layouts differ.".into());
        }

        match self.format {
            BitmapMono | BitmapMonoPacked => {
                let bits: BitVec<u8, Msb0> = self.data.view_bits::<Msb0>()
                    .into_iter()
                    .zip(other.data.view_bits::<Msb0>().into_iter())
                    .map( |(x, y)| *x && *y )
                    .collect();

                let mut result = self.clone();
                result.data = bits.into_vec();
                return Ok(result);
            }
            _ => return Err("Unsupported embedded bitmap format.".into()),
        }
    }
}

impl From<GlyphImage<'_>> for GlyphImageOwned {
    fn from(img: GlyphImage<'_>) -> Self {
        trace!("Cloning bitmap data from a GlyphImage into a GlyphImageOwned for easier data manipulation.");
        let data: Vec<u8> = img.data.into();
        return GlyphImageOwned {
            origin: img.origin,
            width: img.width,
            height: img.height,
            pixels_per_em: img.pixels_per_em,
            data,
            format: img.format,
        };
    }
}

