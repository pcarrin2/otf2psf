use std::fmt::Display;

#derive[Debug]
pub struct Report {
    grapheme: String,
    glyph_type: GlyphType,
    height: u32,
    width: u32,
}

impl Display for Report {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}: {}, {} x {} px", grapheme, glyph_type, height, width);
    }
}

#derive[Debug]
pub enum GlyphType {
    FromEmbeddedBitmap { format: ab_glyph::GlyphImageFormat },
    Rasterized,
}

impl Display for GlyphType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            FromEmbeddedBitmap{format} => write!(f, "TTF embedded bitmap (format: {})", format);
            Rasterized => write!(f, "rasterized from vector font");
        }
    }
}
