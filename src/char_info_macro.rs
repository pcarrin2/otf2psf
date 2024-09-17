use unicode_names;

fn char_info(chr: char) -> String {
    let name = unicode_names::name(chr);
    let value = chr as u32;
    return format!("'{chr}' (U+{value:x}, {name})");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_char_info() {
        let test_char = 'A';
        println!("test char: {}", test_char);
        let output = char_info(test_char);
        println!(output);
    }
}
