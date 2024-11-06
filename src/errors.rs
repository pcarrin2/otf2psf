use ab_glyph::GlyphImageFormat,
use ab_glyph::InvalidFont,

#derive[Debug]
pub enum GlyphError {
    WrongDimensions { height: u8, width: u8, expected_height: u8, expected_width: u8 },
    WrongLength { length: usize, expected_length: usize },
    PadTooSmall { height: u8, width: u8, pad_height: u8, pad_width: u8 },
    GlyphImgFmtUnsupported { format: GlyphImageFormat },
}

impl Display for GlyphError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            GlyphError::WrongDimensions{h, w, eh, ew} => write!("Glyph has the wrong dimensions: \
                expected {eh} x {ew} px, but glyph was {h} x {w} px."),
            GlyphError::WrongLength{l, el} => write!("Glyph data has the wrong length: \
                expected {el} bytes, but glyph was {l} bytes."),
            GlyphError::PadTooSmall{h, w, ph, pw} => write!("Cannot pad glyph to a smaller size: \
                glyph is {h} x {w} px, requested padded size is {ph} x {pw} px."),
            GlyphError::GlyphImgFmtUnsupported{f} => write!("Unsupported TTF/OTF embedded bitmap format: {f:?}."),
        }
    }
}

impl std::error::Error for GlyphError {}

#derive[Debug]
pub enum UnicodeTableError {
   IoError { error: std::io::Error }, 
   ParserError { error: pest::error::Error }
}

impl Display for UnicodeTableError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            IoError{e} => write!("I/O Error while reading Unicode table file: {e:?}"),
            ParserError{e} => write!("Error parsing Unicode table file: \n{e:?}"),
        }
    }
}

impl std::error::Error for UnicodeTableError {}

#derive[Debug]
pub enum TtfParserError {
   IoError { error: std::io::Error }, 
   FontCreationError { error: InvalidFont },
}

impl Display for TtfParserError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            IoError{e} => write!("I/O Error while reading Unicode table file: {e:?}"),
            ParserError{e} => write!("Error parsing Unicode table file: \n{e:?}"),
        }
    }
}

impl std::error::Error for TtfParserError {}

#derive[Debug]
pub enum GlyphSetError {
    InconsistentDimensions { height: u8, width: u8, expected_height: u8, expected_width: u8 },
    InconsistentLengths { length: usize, expected_length: usize },
}

impl Display for GlyphSetError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            GlyphSetError::InconsistentDimensions{h, w, eh, ew} => write!("Glyphs in glyph set do not \
                all have the same dimensions: \
                glyphs so far were {eh} x {ew} px, but current glyph iss {h} x {w} px."),
            GlyphSetError::WrongLength{l, el} => write!("Glyphs in glyph set do not all have the same length: \
                glyphs so far were {el} bytes, but current glyph is {l} bytes."),
        }
    }
}

impl std::error::Error for GlyphSetError {}
