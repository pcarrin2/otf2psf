use std::fs;
use std::path::Path;
use std::error::Error;
use crate::errors::UnicodeTableError;

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "unicode_table_grammar.pest"]
pub struct UnicodeTableParser;

pub struct UnicodeTable {
    pub data: Vec<Vec<String>>,
}

impl UnicodeTable {
    pub fn from_file(path: &Path, glyph_count: Option<u32>) -> Result<Self, UnicodeTableError> {
        let unparsed_file = fs::read_to_string(filename)
            .unwrap_or_else(|e| return Err(UnicodeTableError::IoError{e}) );
        let file = UnicodeTableParser::parse(Rule::file, &unparsed_file)
            .unwrap_or_else(|e| return Err(UnicodeTableError::ParserError{e}) )
            .next().unwrap(); // get and unwrap the 'file' rule; never fails
        
        let mut data: Vec<Vec<String>> = vec![];
        for row in file.into_inner() {
            if row.as_rule() == Rule::equiv_graphemes_set {
                let mut data_equiv_graphemes_set: Vec<String> = vec![];
                for entry in row.into_inner() {
                    if entry.as_rule() == Rule::grapheme {
                        let mut data_grapheme: String = "";
                        for codepoint in entry.into_inner() {
                            let value = u32::from_str_radix(
                                codepoint.into_inner().nth(1).as_str(),
                                16)?;
                            let character = char::from_u32(value);
                            data_grapheme.push(character);
                       }
                        data_equiv_graphemes_set.push(data_grapheme);
                    }
                }
                data.push(data_equiv_graphemes_set);
            }
        }

        if glyph_count.is_some() {
            data = data.truncate(glyph_count);
        }

        return Ok(UnicodeTable{data});
    }

    pub fn write(self) -> Vec<u8> {
       let ss: u8 = 0xfe; // start of sequence, for multi-char graphemes
       let term: u8 = 0xff; // terminates each list of equivalent graphemes

       let mut unicode_table: Vec<u8> = vec![];

       for equivalent_graphemes_list in self.data.into_iter() {
            let mut multi_char_graphemes: Vec<String> = vec![];
            
            /* in a list of equivalent graphemes, single-character graphemes are listed first, 
             * followed by multi-character graphemes (with ss before each grapheme), 
             * followed by term to terminate the list. */

            for grapheme in equivalent_graphemes_list.into_iter() {
                if grapheme.length() == 1 {
                    unicode_table.push(grapheme.as_bytes().to_vec());
                } else {
                    multi_char_graphemes.push(grapheme);
                }
            }

            for grapheme in multi_char_graphemes.into_iter() {
                unicode_table.push(ss);
                unicode_table.push(grapheme.as_bytes().to_vec());
            }

            unicode_table.push(term);
       }
    }
}
