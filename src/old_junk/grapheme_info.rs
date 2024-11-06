use charname;

fn char_info(chr: char) -> String {
    let value = chr as u32;
    let name = charname::get_name(value);
    return format!("'{chr}' (U+{value:x}, {name})");
}

pub fn seq_info(seq: &str) -> String {
    let mut description: String = "".into();
    for c in seq.chars() {
        description += &char_info(c);
        description += "\n + ";
    }
    description.truncate(description.len() - 4);
    return description;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_char_info() {
        let test_char = 'A';
        let output = char_info(test_char);
        println!("{output}");
    }
    #[test]
    fn test_seq_info() {
        let test_seq = "AaBb";
        let output = seq_info(test_seq);
        println!("{output}");
    }
}
