pub fn find_field_ranges(response_data: &[u8]) -> Vec<(usize, usize, String)> {
    let (headers, body) = parse_response_data(response_data);
    let body_start = headers.len();
    let mut field_ranges = Vec::new();

    let field_patterns = [
        (r#""id":([0-9]+)"#, "paymentId"),
        (r#""state":"([^"]+)""#, "state"),
        (
            r#""state":"OUTGOING_PAYMENT_SENT","date":([0-9]+)"#,
            "timestamp",
        ),
        (r#""targetAmount":([0-9\.]+)"#, "targetAmount"),
        (r#""targetCurrency":"([^"]+)""#, "targetCurrency"),
        (r#""targetRecipientId":([0-9]+)"#, "targetRecipientId"),
    ];

    for (pattern, field_name) in field_patterns.iter() {
        if let Ok(regex) = regex::Regex::new(pattern) {
            if let Some(captures) = regex.captures(&body) {
                if let Some(full_match) = captures.get(0) {
                    let start = body_start + full_match.start();
                    let end = body_start + full_match.end();
                    field_ranges.push((start, end, field_name.to_string()));
                }
            }
        }
    }

    field_ranges
}

pub fn find_host_header_range(request_data: &[u8]) -> Option<(usize, usize)> {
    let request_str = String::from_utf8_lossy(request_data);

    if let Ok(regex) = regex::Regex::new(r"host: [^\r\n]+") {
        if let Some(host_match) = regex.find(&request_str) {
            return Some((host_match.start(), host_match.end()));
        }
    }

    None
}

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
