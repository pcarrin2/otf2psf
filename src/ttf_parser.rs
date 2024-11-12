use ab_glyph::{point, PxScale, FontVec, Font, ScaleFont};
use ab_glyph::PxScaleFont;

use bitvec::prelude::*;

use std::path::Path;

use crate::glyph;
use crate::errors::TtfParserError;
use crate::errors::GlyphError;
use crate::report::GlyphReport;
use crate::report::GlyphType;

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
            Some(fg) => { 
                let combined_glyph = char_glyphs.fold(fg, |acc, g| acc.add(g).unwrap());
                Ok(combined_glyph)
            }
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

    pub fn report_char(&self, character: char) -> GlyphReport {
        let glyph_id = self.font.glyph_id(character);
        // check whether this character maps to the same glyph as a known gap in Unicode.
        let glyph_is_undefined = character != '\u{03a2}' && glyph_id == self.font.glyph_id('\u{03a2}');

        let glyph_image = self.font.font.glyph_raster_image2(glyph_id, self.font.height().ceil() as u16);
        let (glyph_type, height, width) = match glyph_image {
            None => {
                let glyph: ab_glyph::Glyph = self.font
                    .glyph_id(character)
                    .with_scale_and_position(self.font.height(), point(0.0, 0.0));
                let width = self.font.h_advance(glyph.id).ceil() as u32;
                let height = self.font.height() as u32;
                (if glyph_is_undefined {GlyphType::Undefined} else {GlyphType::Vector}, height, width)
            }
            Some(g) => (
                if glyph_is_undefined {GlyphType::Undefined} else {GlyphType::EmbeddedBitmap{format: g.format}}, 
                g.height.into(), 
                g.width.into(),
            ),
        };
        return GlyphReport::new(character, glyph_type, height, width);
    }

    
    fn find_embedded_bitmap(&self, character: char) -> Option<glyph::Glyph> {
        let glyph_id = self.font.glyph_id(character);
        let glyph_image = self.font.font.glyph_raster_image2(glyph_id, self.font.height().ceil() as u16)?;
        let glyph = glyph::Glyph::from_glyph_image(glyph_image, character);
        return match glyph {
            Ok(g) => Some(g),
            Err(e) => {
                eprintln!("{e} -- rasterizing instead"); // TODO make this pretty, probably via
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
        let mut pixel_perfect = true;
        
        if let Some(og) = self.font.outline_glyph(glyph) {
            let bounds = og.px_bounds();
            og.draw( |x, y, v| {
                if v != 1.0 && v != 0.0 {
                    pixel_perfect = false;
                }
                // Align this glyph's canvas with the font's baseline. 
                // Warning: glyphs may extend above the font's ascent or below the font's descent
                // -- they will be chopped off in this case. This is, in my opinion, an inherent
                // hazard of smushing an OTF font into a strict monospace bitmap format.
                let y_signed = (y as f32 + bounds.min.y + self.font.ascent()) as i32;
                let x_signed = (x as f32 + bounds.min.x) as i32;

                if y_signed < 0 || x_signed < 0 
                    || y_signed >= height.try_into().unwrap() || x_signed >= width.try_into().unwrap() {
                    eprintln!("While rasterizing {}: pixel ({}, {}) is out of bounds and will not be rendered",
                        character, x_signed, y_signed);
                }

                let y = y_signed as u32;
                let x = x_signed as u32;

                if x < width && y < height && v >= 0.5 {
                    data.set((x as usize) + (y as usize) * (byte_aligned_width as usize), true);
                }
            })
        }

        if !pixel_perfect {
            eprintln!("While rasterizing {}: the glyph outline was not pixel-perfect.", character);
        }

        let data = data.into_vec();
        let grapheme = character.to_string();

        return glyph::Glyph{ height, width, data, grapheme };
        
    }
}
