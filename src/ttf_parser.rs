use ab_glyph::{point, PxScale, FontVec, Font, ScaleFont};
use ab_glyph::PxScaleFont;

use bitvec::prelude::*;

use std::path::Path;

use crate::glyph;
use crate::errors::TtfParserError;
use crate::errors::GlyphError;

/// A parser that creates `Glyph`s from a TTF/OTF font and a character set.
#[derive(Debug)]
pub struct TtfParser {
    /// TTF input font.
    font: PxScaleFont<FontVec>,
}

impl TtfParser {
    pub fn from_font_path(font_path: &Path, height: u32) -> Result<TtfParser, TtfParserError> {
        let font_px_scale = PxScale::from(height as f32);
        let font_data = std::fs::read(font_path)?;
        let font = FontVec::try_from_vec_and_index(font_data, 0)?;
        let scaled_font = font.into_scaled(font_px_scale);
        
        return Ok(Self{font: scaled_font})
    }

    pub fn render_string(&self, grapheme: &str) -> Result<glyph::Glyph, GlyphError> {
        let mut char_glyphs = grapheme.chars().map(|c| self.render_char(c));
        let first_glyph = char_glyphs.nth(0);
        return match first_glyph {
            Some(fg) => { let combined_glyph = char_glyphs.fold(fg, |acc, g| acc.add(g).unwrap()); Ok(combined_glyph)}
            None => Err(GlyphError::EmptyString),
        }
    }

    pub fn render_char(&self, character: char) -> glyph::Glyph {
        let embedded_bitmap = self.find_embedded_bitmap(character);
        return match embedded_bitmap {
            Some(b) => b,
            None => self.rasterize(character),
        }
    }
    
    fn find_embedded_bitmap(&self, character: char) -> Option<glyph::Glyph> {
        let glyph_id = self.font.glyph_id(character);
        let glyph_image = self.font.font.glyph_raster_image2(glyph_id, self.font.height().ceil() as u16)?;
        let glyph = glyph::Glyph::from_glyph_image(glyph_image, character);
        return match glyph {
            Ok(g) => Some(g),
            Err(e) => {
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

        let width = self.font.h_advance(glyph.id).ceil() as u32;
        let height = self.font.height() as u32;
        let byte_aligned_width = (8.0 * (width as f64 / 8.0).ceil()) as u32;

        let mut data = bitvec![u8, Msb0; 0; (byte_aligned_width * height).try_into().unwrap()];
        
        if let Some(og) = self.font.outline_glyph(glyph) {
            let bounds = og.px_bounds();
            og.draw( |x, y, v| {
                let x = (x as f32 + bounds.min.x) as u32;
                let y = (y as f32 + bounds.min.y) as u32;

                if x < width && y < height && v >= 0.5 {
                    data.set((x as usize) + (y as usize) * (byte_aligned_width as usize), true);
                }
            })
        }

        let data = data.into_vec();
        let grapheme = character.to_string();

        return glyph::Glyph{ height, width, data, grapheme };
        
    }
}
