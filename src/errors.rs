use ab_glyph::GlyphImageFormat;
use ab_glyph::InvalidFont;
use std::fmt::Formatter;
use std::fmt::Display;

use crate::unicode_table::Rule;
use std::num::ParseIntError;

#[derive(Debug)]
pub enum GlyphError {
    WrongDimensions { height: u32, width: u32, expected_height: u32, expected_width: u32 },
    WrongLength { length: usize, expected_length: usize },
    PadTooSmall { height: u32, width: u32, pad_height: u32, pad_width: u32 },
    GlyphImgFmtUnsupported { format: GlyphImageFormat },
    EmptyString,
}

impl Display for GlyphError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            GlyphError::WrongDimensions{height, width, expected_height, expected_width} => 
                write!(f, "Glyph has the wrong dimensions: \
                expected {} x {} px, but glyph was {} x {} px.", expected_height, expected_width, height, width),
            GlyphError::WrongLength{length, expected_length} => 
                write!(f, "Glyph data has the wrong length: \
                expected {} bytes, but glyph was {} bytes.", expected_length, length),
            GlyphError::PadTooSmall{height, width, pad_height, pad_width} => 
                write!(f, "Cannot pad glyph to a smaller size: \
                glyph is {} x {} px, requested padded size is {} x {} px.", height, width, pad_height, pad_width),
            GlyphError::GlyphImgFmtUnsupported{format} => 
                write!(f, "Unsupported TTF/OTF embedded bitmap format: {:?}.", format),
            GlyphError::EmptyString => 
                write!(f, "Attempted to render empty string as a glyph."),
        }
    }
}

impl std::error::Error for GlyphError {}

#[derive(Debug)]
pub enum UnicodeTableError {
   IoError { error: std::io::Error }, 
   ParserError { error: pest::error::Error<Rule> },
   InvalidCodepoint { codepoint: u32 },
   ParseIntError { inner: ParseIntError },
}

impl From<ParseIntError> for UnicodeTableError {
    fn from(inner: ParseIntError) -> UnicodeTableError {
        return UnicodeTableError::ParseIntError{inner};
    }
}

impl From<pest::error::Error<Rule>> for UnicodeTableError {
    fn from(e: pest::error::Error<Rule>) -> UnicodeTableError {
        return UnicodeTableError::ParserError{error: e};
    }
}

impl From<std::io::Error> for UnicodeTableError {
    fn from(e: std::io::Error) -> UnicodeTableError {
        return UnicodeTableError::IoError{error: e};
    }
}

impl Display for UnicodeTableError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            UnicodeTableError::IoError{error} => write!(f, "I/O Error while reading Unicode table file: {:?}", error),
            UnicodeTableError::ParserError{error} => write!(f, "Error parsing Unicode table file: \n{:?}", error),
            UnicodeTableError::InvalidCodepoint{codepoint} => write!(f, "U+{:x} is an invalid Unicode codepoint.", codepoint),
            UnicodeTableError::ParseIntError{inner} => write!(f, "Error parsing integer: {:?}", inner),
        }
    }
}

impl std::error::Error for UnicodeTableError {}

#[derive(Debug)]
pub enum TtfParserError {
   IoError { error: std::io::Error }, 
   FontCreationError { error: InvalidFont },
}

impl From<std::io::Error> for TtfParserError {
    fn from(e: std::io::Error) -> TtfParserError {
        return TtfParserError::IoError{error: e};
    }
}

impl From<InvalidFont> for TtfParserError {
    fn from(e: InvalidFont) -> TtfParserError {
        return TtfParserError::FontCreationError{error: e};
    }
}

impl Display for TtfParserError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            TtfParserError::IoError{error} => write!(f, "I/O Error while reading TTF file: {:?}", error),
            TtfParserError::FontCreationError{error} => write!(f, "Error parsing TTF file: \n{:?}", error),
        }
    }
}

impl std::error::Error for TtfParserError {}

#[derive(Debug)]
pub enum GlyphSetError {
    InconsistentDimensions { height: u32, width: u32, expected_height: u32, expected_width: u32 },
    InconsistentLengths { length: usize, expected_length: usize },
    EmptyString
}

impl From<GlyphError> for GlyphSetError {
    fn from(e: GlyphError) -> GlyphSetError {
        match e {
            GlyphError::EmptyString => return GlyphSetError::EmptyString,
            // TODO make not panic
            _ => panic!("Casting wrong variant of GlyphError to GlyphSetError"),
        }
    }
}

impl Display for GlyphSetError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            GlyphSetError::InconsistentDimensions{height, width, expected_height, expected_width} => 
                write!(f, "Glyphs in glyph set do not all have the same dimensions: \
                glyphs so far were {} x {} px, but current glyph is {} x {} px.", expected_height, expected_width, height, width),
            GlyphSetError::InconsistentLengths{length, expected_length} => 
                write!(f, "Glyphs in glyph set do not all have the same length: \
                glyphs so far were {} bytes, but current glyph is {} bytes.", expected_length, length),
            GlyphSetError::EmptyString => 
                write!(f, "Attempted to render empty string as a glyph."),
        }
    }
}

impl std::error::Error for GlyphSetError {}
