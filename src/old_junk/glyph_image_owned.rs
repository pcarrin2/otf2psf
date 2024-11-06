use std::error;
use ab_glyph::{Point, FontRef, Font, ScaleFont, Glyph, GlyphImageFormat, GlyphImageFormat::*};
use ab_glyph::point;
use ab_glyph::PxScaleFont;
use ab_glyph::v2::GlyphImage;
use itertools::Itertools;
use bitvec::prelude::*;
use bitvec::vec::BitVec;
use std::iter::once;
use log::{Level, log, warn, debug, trace};
use crate::glyph_image_pixel::GlyphImagePixel;
use image::Pixel;

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

pub enum GlyphImagePixelFormat {
    Mono,
    Gray2,
    Gray4,
    Gray8,
    PremulBgra32,
}

impl GlyphImagePixelFormat {
    fn length(self) => Result<usize, Box<dyn error::Error>> {
        return match self {
            Self::Mono => Ok(1),
            Self::Gray2 => Ok(2),
            Self::Gray4 => Ok(4),
            Self::Gray8 => Ok(8),
            Self::PremulBgra32 => Ok(32),
            _ => Err("Unsupported glyph image pixel format.".into())
        }
    }
    fn from_image_format(image_format: GlyphImageFormat) -> Result<Self, Box<dyn error::Error>> {
        return match image_format {
            BitmapMono | BitmapMonoPacked => Ok(Mono),
            BitmapGray2 | BitmapGray2Packed => Ok(Gray2),
            BitmapGray4 | BitmapGray4Packed => Ok(Gray4),
            BitmapGray8 => Ok(Gray8),
            BitmapPremulBgra32 => Ok(PremulBgra32),
            _ => Err("Unsupported glyph image format.".into())
        }
    }
}

pub struct GlyphImagePixel {
   format: GlyphImagePixelFormat,
   data: BitVec,
}

impl GlyphImagePixel {
    fn new(format: GlyphImagePixelFormat, data: BitVec) -> Result<Self, Box<dyn error::Error>> {
        if data.len() == format.length() {
            return Ok(Self{ format, data })
        } else {
            return Err("Pixel data is the wrong length for given format".into())
        }
    }

    fn data(self) -> BitVec {
        return self.data
    }
    
    fn format(self) -> GlyphImagePixelFormat {
        return self.format
    }

    fn convert_format(self, target_format: GlyphImagePixelFormat) -> Result<Self, Box<dyn error::Error>> {
        match self.format {
            GlyphImagePixelFormat::Mono => {
                let data: BitVec = bitvec![self.data[0]?; target_format.length()];
                return Ok(Self::new(target_format, data))?)
                }
            _ => return Err("Unsupported pixel format conversion.")
            }
        }
    }
}

/// The horizontal edge that a glyph is anchored to when it's padded to fill a larger canvas. If
/// the "Middle" option is chosen, the glyph will be horizontally centered in the large canvas -- 
/// though it may end up a half-pixel left of true center, if the difference between glyph width 
/// and canvas width is odd.
///
/// Used in `GlyphImageOwned::pad_to_dimensions()`.
pub enum PaddingHorizontalAnchor {
    pub Left,
    pub Middle,
    pub Right,
}


/// The vertical edge that a glyph is anchored to when it's padded to fill a larger canvas. If
/// the "Middle" option is chosen, the glyph will be vertically centered in the large canvas -- 
/// though it may end up a half-pixel above true center, if the difference between glyph height 
/// and canvas height is odd.
///
/// Used in `GlyphImageOwned::pad_to_dimensions()`.
pub enum PaddingVerticalAnchor {
    pub Top,
    pub Middle,
    pub Bottom,
}

pub struct PaddingDirection {
    pub horizontal: PaddingHorizontalAnchor,
    pub vertical: PaddingVerticalAnchor,
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
        trace!("Embedded bitmap found for character {:?}", character);
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

    /// Creates a `GlyphImageOwned` with an empty (zeroed-out) data field of the right length,
    /// given a format and layout information. The resulting object can be used as a "blank canvas"
    /// for further operations.
    pub fn empty(origin: Point, width: u16, height: u16, pixels_per_em: u16, format: GlyphImageFormat) 
        -> Result<Self, Box<dyn error::Error>> 
    {
        let mut image = GlyphImageOwned {
            origin,
            width,
            height,
            pixels_per_em,
            data: vec![],
            format,
        };

        match format {
            BitmapMono | BitmapMonoPacked => {
                let data_length_bits = initial_image.padded_width() * height;
                let data = bitvec![u8, Msb0; 0; data_length_bits].into_vec();
                image.data = data;                
            }
            _ => return Err("Unsupported glyph image format.".into())
        }

        return Ok(data);
    }

    pub fn pixel_format(self) -> Result<GlyphImagePixelFormat, Box<dyn error::Error>> {
        return GlyphImagePixelFormat::from_image_format(self.format)?
    }


    pub fn pixel_length(self) -> Result<usize, Box<dyn error::Error>> {
        return self.pixel_format()?.length()?
    }

    pub fn pixels(self) -> Result<Vec<Vec<GlyphImagePixel>>, Box<dyn error::Error>> {
        let pixels: Vec<Vec<GlyphImagePixel>> = self.data.view_bits()
            .chunks_exact(self.padded_width())
            .map(
                    |row| row[0..(self.pixel_length()*self.width)]
                    .chunks_exact(self.pixel_length())
                    .map( |bits| GlyphImagePixel::new(self.pixel_format, bits) )
                    .collect::Vec<GlyphImagePixel>()
                );
        return pixels;

    }

    pub fn from_pixels
        (
        pixels: Vec<Vec<GlyphImagePixel>>, 
        origin: Point, 
        pixels_per_em: u16, 
        format: GlyphImageFormat
        ) -> Result<Self, Box<dyn error::Error>> 
    {
        if let px = pixels[0][0] {
            if px.format() != format.pixel_format() {
                return Err("Target image format doesn't match pixel format.")
            }
        }

        let height = pixels.len();
        let width = 

        let px_format = format.pixel_format();
        let padding_px = GlyphImagePixel::new(px_format, bitvec![u8, Msb0; 0; px_format.length()])

        for row in pixels.iter() {
            let row_bits: Vec<u8> = row.iter()
                .pad_using()
                .map( |px| px.data() )
                .
        }
    }

    /// If the glyph image's `format` is a byte-boundary-padded type (eg `BitmapMono`), returns the
    /// length of a row in pixel-sized chunks, including the padding. If the `format` is a 
    /// non-padded type (eg `BitmapMonoPacked`), simply returns the glyph image's width.
    pub fn padded_width(self) -> u16 {
        match self.format {
            BitmapMono => {return (self.width as f32 / 8.0).ceil() as u16 * 8};
            BitmapGray2 => {return (self.width as f32 / 4.0).ceil() as u16 * 4};
            BitmapGray4 => {return (self.width as f32 / 2.0).ceil() as u16 * 2};
            BitmapMonoPacked 
                | BitmapGray2Packed
                | BitmapGray4Packed
                | BitmapGray8
                | BitmapPremulBgra32
                => { return self.width; }
        }
    }

    pub fn convert_format(self, target_format: &GlyphImageFormat) -> Result<Self, Box<dyn error::Error>> {
        trace!("Converting a {:?} glyph to the {:?} format.", self.format, fmt);
        trace!("Original glyph dimensions: height = {}, width = {}, length = {}",
            self.height, self.width, self.data.len());
        let mut converted: GlyphImageOwned = GlyphImageOwned::empty(
            self.origin,
            self.width,
            self.height,
            self.pixels_per_em,
            fmt
            );

        let target_pixel_format = converted.pixel_format();

        let mut converted_pixels = self.pixels()?;
        for row in converted_pixels.iter() {
            for px in row.iter() {
                px = px.convert_format(target_pixel_format);
            }
        }

        for (src_row, dest_row) in self.pixels()?.into_iter()
            .zip(converted.pixels_mut()?.into_iter_mut()) {
            for (src_px, dest_px) in src_row.into_iter()
                .zip(dest_row.into_iter_mut()) {
                dest_px = src_px.convert_format(target_pixel_format);
            }
        }
        
        let conv_length = converted.data.len();
        trace!("Converted glyph has a length of {conv_length} bytes");
        return Ok(converted)

    }


    /// Pads a GlyphImageOwned to larger dimensions with empty space.
    ///
    /// The `h_anchor` and `v_anchor` options determine how the existing glyph image is anchored 
    /// on the larger canvas. For example, if `h_anchor` is `Middle` and `v_anchor` is `Top`, 
    /// then the existing glyph will be anchored to the top-center of the new canvas, with padding 
    /// applied below it and to both sides.
    pub fn pad_to_dimensions(self, padded_height: u16, padded_width: u16, 
        h_anchor: PaddingHorizontalAnchor, v_anchor: PaddingVerticalAnchor ) 
        -> Result<Self, Box<dyn error::Error>> {
            match self.format {
                BitmapMono => {

                }

                BitmapMonoPacked => {
                    let new_canvas = bitvec![u8, Msb0; 0; (padded_width * padded_height).into()];
                    let row_range = match v_anchor {
                        Top => 0..self.height,
                        Middle => {
                            let v_midpoint = padded_height / 2;
                            let top_row = v_midpoint - (self.height / 2);
                            let bottom_row = top_row + self.height;
                            bottom_row..top_row
                        }
                        Bottom => (padded_height - self.height)..padded_height,
                    };

                    let col_range = match h_anchor {
                        Left => 0..self.width,
                        Middle => {
                            let h_midpoint = padded_width / 2;
                            let left_col = h_midpoint - (self.width / 2);
                            let right_col = left_col + self.width;
                            left_col..right_col
                        }
                        Bottom => (padded_width - self.width)..padded_width,
                    };


                }

                _ => {return Err("Unsupported glyph image format.")}
            }
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

    pub fn draw_to_ascii_art(self, log_level: Level) -> Result<(), Box<dyn error::Error>> {
        let h = self.height;
        let w = self.width;
        let l = self.data.len();
        let image = self.convert_format(&BitmapMonoPacked)?;
        log!(log_level, "ASCII art of glyph image (height {h}, width {w}, length {l}):");
        let image_data_bits = image.data.view_bits::<Msb0>();
        for row in image_data_bits.chunks(image.width.into()) {
            let ascii_row: String = row
                .iter()
                .map( |b| {if *b {['#', '#']} else {['.', '.']} } )
                .flatten()
                .collect();
            log!(log_level, "{}", ascii_row);
        }
        Ok(())
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

