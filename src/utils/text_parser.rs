use tracing::info;

use crate::{
    domain::Provider,
    utils::patterns::{HOST_HEADER_PATTERN, get_field_patterns},
};

pub fn find_field_ranges(response_data: &[u8], provider: &Provider) -> Vec<(usize, usize)> {
    let (headers, body) = parse_response_data(response_data);
    let body_start = headers.len();
    let mut field_ranges = Vec::new();

    for (pattern, field_name) in get_field_patterns(provider).iter() {
        if let Ok(regex) = regex::Regex::new(pattern) {
            if let Some(captures) = regex.captures(&body) {
                if let Some(full_match) = captures.get(0) {
                    let start = body_start + full_match.start();
                    let end = body_start + full_match.end();
                    field_ranges.push((start, end));
                    info!(
                        "     ✅ Found {}: {} (Bytes {}..{})",
                        field_name,
                        full_match.as_str(),
                        start,
                        end
                    );
                }
            }
        }
    }

    field_ranges
}

pub fn find_host_header_range(request_data: &[u8]) -> Option<(usize, usize)> {
    let request_str = String::from_utf8_lossy(request_data);

    if let Ok(regex) = regex::Regex::new(HOST_HEADER_PATTERN) {
        if let Some(host_match) = regex.find(&request_str) {
            info!(
                "     ✅ Found host header: range {}..{}",
                host_match.start(),
                host_match.end()
            );
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
    (String::new(), response_str.to_string())
}
