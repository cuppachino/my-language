pub fn is_valid_identifier(s: &str) -> bool {
    let legal_first_chars = vec!['_', '$'];
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if !first.is_alphabetic() && !legal_first_chars.contains(&first) {
        return false;
    }
    for c in chars {
        if !c.is_alphanumeric() && !legal_first_chars.contains(&c) {
            return false;
        }
    }
    true
}
