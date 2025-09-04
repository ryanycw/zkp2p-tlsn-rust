use serde_json;
use std::collections::HashMap;

/// Parse HTTP headers from raw response data
pub fn parse_http_headers(
    response_data: &[u8],
) -> Result<(HashMap<String, String>, usize), Box<dyn std::error::Error>> {
    let response_str = String::from_utf8_lossy(response_data);
    let mut headers = HashMap::new();
    let header_end;

    // Find the end of headers (double CRLF)
    if let Some(pos) = response_str.find("\r\n\r\n") {
        header_end = pos + 4;
    } else if let Some(pos) = response_str.find("\n\n") {
        header_end = pos + 2;
    } else {
        return Err("Could not find end of HTTP headers".into());
    }

    let header_section = &response_str[..header_end];
    let lines: Vec<&str> = header_section.lines().collect();

    // Skip the status line
    for line in lines.iter().skip(1) {
        if let Some(colon_pos) = line.find(':') {
            let name = line[..colon_pos].trim().to_lowercase();
            let value = line[colon_pos + 1..].trim().to_string();
            headers.insert(name, value);
        }
    }

    Ok((headers, header_end))
}

/// Decode chunked HTTP response body with enhanced error handling and logging
pub fn decode_chunked_body(chunked_data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    let mut pos = 0;
    let chunked_str = String::from_utf8_lossy(chunked_data);
    let chunked_bytes = chunked_data;
    let mut chunk_count = 0;
    const MAX_CHUNKS: usize = 1000; // Safety limit

    println!(
        "   🔧 Decoding chunked body ({} bytes)...",
        chunked_data.len()
    );

    while pos < chunked_bytes.len() && chunk_count < MAX_CHUNKS {
        chunk_count += 1;

        // Find the chunk size line (ends with CRLF)
        let remaining = &chunked_str[pos..];
        let size_line_end = if let Some(crlf_pos) = remaining.find("\r\n") {
            crlf_pos
        } else if let Some(lf_pos) = remaining.find('\n') {
            lf_pos
        } else {
            // No more chunk size lines found
            break;
        };

        let size_line = &remaining[..size_line_end];

        // Parse chunk size (hexadecimal)
        let chunk_size_str = if let Some(semicolon_pos) = size_line.find(';') {
            // Remove chunk extensions after semicolon
            &size_line[..semicolon_pos]
        } else {
            size_line
        }
        .trim();

        // Validate chunk size string is not empty
        if chunk_size_str.is_empty() {
            return Err("Empty chunk size found".into());
        }

        let chunk_size = usize::from_str_radix(chunk_size_str, 16)
            .map_err(|e| format!("Invalid chunk size '{}': {}", chunk_size_str, e))?;

        println!("     Chunk {}: {} bytes", chunk_count, chunk_size);

        if chunk_size == 0 {
            // Final chunk, stop processing
            println!(
                "   ✅ Final chunk reached, decoded {} chunks total",
                chunk_count
            );
            break;
        }

        // Move past the size line and CRLF
        pos += size_line_end + if remaining.contains("\r\n") { 2 } else { 1 };

        // Validate we have enough data for this chunk
        if pos + chunk_size > chunked_bytes.len() {
            return Err(format!(
                "Chunk {} claims {} bytes but only {} bytes available from position {}",
                chunk_count,
                chunk_size,
                chunked_bytes.len() - pos,
                pos
            )
            .into());
        }

        // Extract chunk data
        result.extend_from_slice(&chunked_bytes[pos..pos + chunk_size]);
        pos += chunk_size;

        // Skip trailing CRLF after chunk data
        if pos < chunked_bytes.len() && chunked_bytes[pos] == b'\r' {
            pos += 1;
        }
        if pos < chunked_bytes.len() && chunked_bytes[pos] == b'\n' {
            pos += 1;
        }
    }

    if chunk_count >= MAX_CHUNKS {
        return Err(format!("Too many chunks ({}), possible infinite loop", chunk_count).into());
    }

    println!(
        "   ✅ Successfully decoded {} bytes from {} chunks",
        result.len(),
        chunk_count
    );
    Ok(result)
}

/// Parse JSON from decoded HTTP response body with enhanced error handling
pub fn parse_json_response(
    body_data: &[u8],
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    if body_data.is_empty() {
        return Err("Empty response body".into());
    }

    let body_str = String::from_utf8_lossy(body_data);
    let trimmed_body = body_str.trim();

    if trimmed_body.is_empty() {
        return Err("Response body contains only whitespace".into());
    }

    // Log first and last 100 characters for debugging (without revealing sensitive data)
    let preview_len = 100.min(trimmed_body.len());
    println!(
        "   🔍 JSON body preview: {}...",
        &trimmed_body[..preview_len]
    );

    if trimmed_body.len() > preview_len {
        let suffix_start = trimmed_body.len() - preview_len;
        println!(
            "   🔍 JSON body suffix: ...{}",
            &trimmed_body[suffix_start..]
        );
    }

    match serde_json::from_str(trimmed_body) {
        Ok(json_value) => {
            println!("   ✅ JSON parsed successfully");
            Ok(json_value)
        }
        Err(e) => {
            println!("   ❌ JSON parse error: {}", e);

            // Try to identify the issue
            if !trimmed_body.starts_with('{') && !trimmed_body.starts_with('[') {
                return Err(
                    "Response body does not appear to be JSON (missing opening brace/bracket)"
                        .into(),
                );
            }

            Err(format!("JSON parse error: {}", e).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunked_decoding_simple() {
        // Sample chunked HTTP body: "Hello, World!" split into two chunks
        let chunked_body = b"7\r\nHello, \r\n6\r\nWorld!\r\n0\r\n\r\n";

        let result = decode_chunked_body(chunked_body).unwrap();
        let decoded = String::from_utf8(result).unwrap();
        assert_eq!(decoded, "Hello, World!");
    }

    #[test]
    fn test_chunked_decoding_single_chunk() {
        // Single chunk with final zero chunk
        let chunked_body = b"D\r\nHello, World!\r\n0\r\n\r\n";

        let result = decode_chunked_body(chunked_body).unwrap();
        let decoded = String::from_utf8(result).unwrap();
        assert_eq!(decoded, "Hello, World!");
    }

    #[test]
    fn test_chunked_decoding_with_extensions() {
        // Chunk with extensions (after semicolon)
        let chunked_body = b"7;charset=utf-8\r\nHello, \r\n6\r\nWorld!\r\n0\r\n\r\n";

        let result = decode_chunked_body(chunked_body).unwrap();
        let decoded = String::from_utf8(result).unwrap();
        assert_eq!(decoded, "Hello, World!");
    }

    #[test]
    fn test_chunked_decoding_empty() {
        // Empty chunked body (just final chunk)
        let chunked_body = b"0\r\n\r\n";

        let result = decode_chunked_body(chunked_body).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_json_parsing_valid() {
        let json_data = br#"{"primaryAmount": "100.00", "currency": "USD", "status": "COMPLETED"}"#;

        let json_value = parse_json_response(json_data).unwrap();
        assert_eq!(
            json_value.get("primaryAmount").and_then(|v| v.as_str()),
            Some("100.00")
        );
        assert_eq!(
            json_value.get("currency").and_then(|v| v.as_str()),
            Some("USD")
        );
        assert_eq!(
            json_value.get("status").and_then(|v| v.as_str()),
            Some("COMPLETED")
        );
    }

    #[test]
    fn test_json_parsing_empty() {
        let json_data = b"";

        let result = parse_json_response(json_data);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Empty response body")
        );
    }

    #[test]
    fn test_json_parsing_invalid() {
        let json_data = b"Not JSON data";

        let result = parse_json_response(json_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_http_header_parsing() {
        let http_response = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\nServer: nginx/1.18.0\r\n\r\nBody content here";

        let (headers, body_start) = parse_http_headers(http_response).unwrap();
        assert_eq!(
            headers.get("content-type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(
            headers.get("transfer-encoding"),
            Some(&"chunked".to_string())
        );
        assert_eq!(headers.get("server"), Some(&"nginx/1.18.0".to_string()));
        assert!(body_start > 0);

        // Verify body starts at correct position
        let body = &http_response[body_start..];
        assert!(body.starts_with(b"Body content here"));
    }

    #[test]
    fn test_http_header_parsing_no_crlf() {
        let http_response = b"HTTP/1.1 200 OK\nContent-Type: application/json\nTransfer-Encoding: chunked\n\nBody content";

        let (headers, body_start) = parse_http_headers(http_response).unwrap();
        assert_eq!(
            headers.get("content-type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(
            headers.get("transfer-encoding"),
            Some(&"chunked".to_string())
        );
        assert!(body_start > 0);
    }

    #[test]
    fn test_http_header_parsing_malformed() {
        let http_response = b"HTTP/1.1 200 OK\nNo double newline separator";

        let result = parse_http_headers(http_response);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Could not find end of HTTP headers")
        );
    }

    #[test]
    fn test_chunked_decoding_invalid_size() {
        // Invalid hex chunk size
        let chunked_body = b"ZZ\r\nHello, \r\n0\r\n\r\n";

        let result = decode_chunked_body(chunked_body);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid chunk size")
        );
    }

    #[test]
    fn test_chunked_decoding_insufficient_data() {
        // Claims 15 bytes (0xF = 15) but body cuts off after "Hello" (incomplete chunk data)
        let chunked_body = b"F\r\nHello"; // Missing rest of chunk data and final chunk

        let result = decode_chunked_body(chunked_body);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("bytes available"));
    }
}
