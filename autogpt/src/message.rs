/// Parses a key-value formatted payload string into individual values.
///
/// The function expects a string in the format:
/// `input=some text;language=python`, and extracts the `input` and `language` fields.
///
/// # Arguments
///
/// * `payload` - A string containing key-value pairs separated by semicolons.
///
/// # Returns
///
/// * A tuple `(input, language)` extracted from the payload. Defaults are empty string and "python" respectively
///   if the keys are not present.
pub fn parse_kv(payload: &str) -> (String, String) {
    let mut input = "".to_string();
    let mut lang = "python".to_string();

    for part in payload.split(';') {
        let mut kv = part.splitn(2, '=');
        let key = kv.next().unwrap_or("");
        let val = kv.next().unwrap_or("").to_string();
        if key == "input" {
            input = val;
        } else if key == "language" {
            lang = val;
        }
    }

    (input, lang)
}
