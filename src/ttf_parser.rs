use ab_glyph::{Point, PxScale, FontRef, Font, ScaleFont, Glyph, GlyphImageFormat, GlyphImageFormat::*};
use ab_glyph::PxScaleFont;
use ab_glyph::v2::GlyphImage;
use bitvec::prelude::*;
use bitvec::vec::BitVec;
use std::path::Path;

use crate::glyph;
use crate::errors::TtfParserError;

/// A parser that creates `Glyph`s from a TTF/OTF font and a character set.
#[derive(Debug)]
pub struct TtfParser {
    /// TTF input font.
    font: PxScaleFont<FontRef>,
    /// A character set: each element is a list of Unicode graphemes that should map to the same
    /// glyph in the final PSF2 font. The first grapheme from each list is taken as a "reference",
    /// meaning that it is rendered to produce the PSF2 glyph for the whole list.
    charset: Vec<Vec<String>>,
}

impl TtfParser {
    pub fn from_font_and_charset_paths(font_path: &Path, charset_path: &Path, height: u32) 
        -> Result<Self, TtfParserError> {
        let font_px_scale = PxScale::from(height as f32);
        let font_data = std::fs::read(font_path)
            .unwrap_or_else(|e| return Err(TtfParserError::IoError{e}));
        let font = FontRef::try_from_slice(&font_data)
            .unwrap_or_else(|e| return Err(TtfParserError::FontCreationError{e}));

        let charset = UnicodeTable::from_file(uc_table_path);

        return Ok(Self{font, charset})
    }

    pub fn render_string(&self, grapheme: &str) -> glyph::Glyph {
        let char_glyphs = grapheme.iter().map(render_char);
        let first_glyph = char_glyphs.nth(0);
        let combined_glyph = char_glyphs.fold(first_glyph, |acc, g| acc.add(g));
        return combined_glyph;
    }

    pub fn render_char(&self, character: char) -> glyph::Glyph {
        let embedded_bitmap = self.find_embedded_bitmap(character);
        return match embedded_bitmap {
            Some(b) => b
            None => self.rasterize(character)
        }
    }
    
    fn find_embedded_bitmap(&self, character: char) -> Option<glyph::Glyph> {
        let glyph_id = self.font.glyph_id(character);
        let glyph_image = self.font.font.glyph_raster_image2(glyph_id, self.font.height().ceil() as u16)?;
        let glyph = glyph::Glyph::from_glyph_image(glyph_image, character);
        return match glyph {
            Ok(g) => Some(g),
            GlyphError(e) => {
                eprintln!("{e:?} -- rasterizing instead"); // TODO make this pretty, probably via
                                                           // logging.
                None
            }
        }
    }

    fn rasterize(&self, character: char) -> glyph::Glyph {
        let glyph: ab_glyph::Glyph = self.font
            .glyph_id(character)
            .with_scale_and_position(self.font.height(), point(0.0, 0.0));

        let width = self.font.h_advance(glyph.id).ceil() as u8;
        let height = self.font.height() as u8;
        let byte_aligned_width = (8 * (width / 8.0).ceil()) as u8;

        let mut data = bitvec![u8, Msb0; 0; (byte_aligned_width * height).into()];
        
        if let Some(og) = font.outline_glyph(glyph) {
            let bounds = og.px_bounds();
            og.draw( |x, y, v| {
                let x = (x as f32 + bounds.min.x) as u16;
                let y = (y as f32 + bounds.min.y) as u16;

                if x < width && y < height && v >= 0.5 {
                    data.set((x as usize) + (y as usize) * (byte_aligned_width as usize), true);
                }
            })
        }

        let data = data.into_vec();

        return glyph::Glyph{ height, width, data, grapheme };
        
    }
}
