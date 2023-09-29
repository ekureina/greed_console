use std::collections::HashMap;

/// Dumb conversion from Roman String to optional usize
pub fn from_roman(roman_text: &str) -> Option<usize> {
    let mut roman_map: HashMap<&str, usize> = HashMap::new();
    roman_map.insert("I", 1);
    roman_map.insert("II", 2);
    roman_map.insert("III", 3);
    roman_map.insert("IV", 4);
    roman_map.insert("V", 5);

    roman_map.get(roman_text).copied()
}
