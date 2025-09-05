pub fn parse_response_data(response_data: &[u8]) -> (String, String) {
    let response_str = String::from_utf8_lossy(response_data);

    // Use regex to find the end of HTTP headers (double CRLF or double LF)
    let separator = regex::Regex::new(r"\r\n\r\n|\n\n").unwrap();
    if let Some(mat) = separator.find(&response_str) {
        let header_section = response_str[..mat.end()].to_string();
        let body_section = response_str[mat.end()..].to_string();
        return (header_section, body_section);
    }

    // Fallback: return entire response as header if no separator found
    (response_str.to_string(), String::new())
}
