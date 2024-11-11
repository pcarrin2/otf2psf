use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug)]
pub struct Report {
    character: char,
    glyph_type: GlyphType,
    height: u32,
    width: u32,
}

impl Report {
    pub fn new(character: char, glyph_type: GlyphType, height: u32, width: u32) -> Self {
        return Self{character, glyph_type, height, width}
    }
}

impl Display for Report {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}: {}, {} x {} px", self.character, self.glyph_type, self.height, self.width)
    }
}

#[derive(Debug)]
pub enum GlyphType {
    EmbeddedBitmap { format: ab_glyph::GlyphImageFormat },
    Vector,
}

impl Display for GlyphType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::EmbeddedBitmap{format} => write!(f, "TTF embedded bitmap (format: {:?})", format),
            Self::Vector => write!(f, "Vector glyph"),
        }
    }
}
