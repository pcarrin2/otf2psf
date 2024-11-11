use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug)]
pub struct Report {
    grapheme: String,
    glyph_type: GlyphType,
    height: u32,
    width: u32,
}

impl Report {
    fn new(grapheme: String, glyph_type: GlyphType, height: u32, width: u32) -> Self {
        return Self{grapheme, glyph_type, height, width}
    }
}

impl Display for Report {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}: {}, {} x {} px", self.grapheme, self.glyph_type, self.height, self.width)
    }
}

#[derive(Debug)]
pub enum GlyphType {
    FromEmbeddedBitmap { format: ab_glyph::GlyphImageFormat },
    Rasterized,
}

impl Display for GlyphType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::FromEmbeddedBitmap{format} => write!(f, "TTF embedded bitmap (format: {:?})", format),
            Self::Rasterized => write!(f, "rasterized from vector font"),
        }
    }
}
