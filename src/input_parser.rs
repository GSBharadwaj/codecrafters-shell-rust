
pub fn parse(input: &String) -> Vec<String> {
    input.as_str()
        .split_whitespace()
        .into_iter()
        .map(str::trim)
        .map(str::to_string)
        .collect()
}
