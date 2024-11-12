use std::fmt::Display;
use std::fmt::Formatter;
use std::path::Path;
use crate::ttf_parser::TtfParser;
use crate::unicode_table::UnicodeTable;
use unicode_blocks::UnicodeBlock;

#[derive(Debug)]
pub struct GlyphReport {
    character: char,
    glyph_type: GlyphType,
    height: u32,
    width: u32,
}

impl GlyphReport {
    pub fn new(character: char, glyph_type: GlyphType, height: u32, width: u32) -> Self {
        return Self{character, glyph_type, height, width}
    }
}

impl Display for GlyphReport {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let pretty_unicode = format!("U+{:04x}", u32::from(self.character));
        write!(f, "{} ({}): {}, {} x {} px", 
            self.character, 
            pretty_unicode, 
            self.glyph_type, 
            self.height, 
            self.width,
            )
    }
}

#[derive(Debug)]
pub enum GlyphType {
    EmbeddedBitmap { format: ab_glyph::GlyphImageFormat },
    Vector,
    Undefined,
}

impl Display for GlyphType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::EmbeddedBitmap{format} => write!(f, "TTF embedded bitmap (format: {:?})", format),
            Self::Vector => write!(f, "Vector glyph"),
            Self::Undefined => write!(f, "Not found in font")
        }
    }
}


pub fn report_char_vec(ttf_parser: TtfParser, characters: Vec<char>) -> () {
    for c in characters.into_iter() {
        println!("{}", ttf_parser.report_char(c));
    }
}

pub fn report_unicode_block(ttf_parser: TtfParser, block: UnicodeBlock) -> () {
    let block_characters: Vec<char> = (block.start() .. block.end())
        .map(|i| char::from_u32(i).unwrap()).collect();
    report_char_vec(ttf_parser, block_characters);
}

pub fn report_unicode_table(ttf_parser: TtfParser, unicode_table_file: &Path) 
    -> Result<(), Box<dyn std::error::Error>> {
    let unicode_table = UnicodeTable::from_file(unicode_table_file, None)?;
    // list of equiv graphemes has already been sorted by length, so the zeroth/reference grapheme 
    // will be single-character if possible
    let chars_to_report: Vec<char> = unicode_table.data.into_iter()
        .map(|row| row[0].clone()) // acquire reference grapheme for each set of equiv graphemes
        .fold(String::new(), |acc, reference_grapheme| acc + &reference_grapheme)
        .chars().collect();

    Ok(report_char_vec(ttf_parser, chars_to_report))
}
