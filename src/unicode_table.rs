use std::fs;
use std::path::Path;

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
        let unparsed_file = fs::read_to_string(path)?;
        let file = UnicodeTableParser::parse(Rule::file, &unparsed_file)?
            .next().unwrap(); // get and unwrap the 'file' rule; never fails
        
        let mut data: Vec<Vec<String>> = vec![];
        for row in file.into_inner() {
            if row.as_rule() == Rule::equiv_graphemes_set {
                let mut data_equiv_graphemes_set: Vec<String> = vec![];
                for entry in row.into_inner() {
                    if entry.as_rule() == Rule::grapheme {
                        let mut data_grapheme: String = String::new();
                        for codepoint in entry.into_inner() {
                            let value = u32::from_str_radix(
                                codepoint.into_inner().nth(1)
                                .expect("Unicode 'U+' prefix without codepoint found in Unicode table").as_str(),
                                16)?;
                            let character = char::from_u32(value);
                            match character {
                                None => return Err(UnicodeTableError::InvalidCodepoint{codepoint: value}),
                                Some(c) => {eprintln!("pushing {} to grapheme", c); data_grapheme.push(c)}
                            }
                       }
                        data_equiv_graphemes_set.push(data_grapheme);
                    }
                }
                /* list single-character graphemes first */
                data_equiv_graphemes_set.sort_by_key(|str| str.chars().count());
                eprintln!("sorted: {:?}", data_equiv_graphemes_set);
                data.push(data_equiv_graphemes_set);
            }
        }

        if let Some(gc) = glyph_count {
            data.truncate(gc as usize);
        }
        return Ok(UnicodeTable{data});
    }

    pub fn write(self) -> Vec<u8> {
       let ss: u8 = 0xfe; // start of sequence, for multi-char graphemes
       let term: u8 = 0xff; // terminates each list of equivalent graphemes

       let mut unicode_table: Vec<u8> = vec![];

       for equivalent_graphemes_list in self.data.into_iter() {
            /* In a list of equivalent graphemes, single-character graphemes are listed first, 
             * followed by multi-character graphemes (with ss before each grapheme), 
             * followed by term to terminate the list. Since we have already sorted equivalent
             * graphemes by length when constructing the Unicode table, we don't have to re-sort
             * them now. */
            for grapheme in equivalent_graphemes_list.into_iter() {
                if grapheme.chars().count() == 1 {
                    eprintln!("single char grapheme {}", grapheme);
                    unicode_table.extend(grapheme.as_bytes().to_vec());
                } else {
                    eprintln!("multi char grapheme {}", grapheme);
                    unicode_table.push(ss);
                    unicode_table.extend(grapheme.as_bytes().to_vec());
                }
            }
            unicode_table.push(term);
       }
       return unicode_table;
    }
}
