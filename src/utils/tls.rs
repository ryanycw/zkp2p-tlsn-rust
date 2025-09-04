use http_body_util::Empty;
use hyper::{Request, body::Bytes};

/// Builds an HTTP request with common headers for TLSNotary attestation
pub fn build_request(
    uri: &str,
    server_name: &str,
    extra_headers: &[(&str, &str)],
    description: &str,
    user_agent: &str,
) -> Result<Request<Empty<Bytes>>, Box<dyn std::error::Error>> {
    println!("📡 Building request: {}", description);
    println!("   Endpoint: {}", uri);

    let mut request_builder = Request::builder()
        .uri(uri)
        .header("Host", server_name)
        .header("Accept", "*/*")
        // Using "identity" instructs the Server not to use compression for its HTTP response.
        // TLSNotary tooling does not support compression.
        .header("Accept-Encoding", "identity")
        .header("Connection", "close")
        .header("User-Agent", user_agent);

    for (key, value) in extra_headers {
        request_builder = request_builder.header(*key, *value);
        if !key.eq_ignore_ascii_case("Cookie") && !key.eq_ignore_ascii_case("X-Access-Token") {
            println!("   Header: {}: {}", key, value);
        } else {
            println!("   Header: {}: [REDACTED]", key);
        }
    }

    Ok(request_builder.body(Empty::<Bytes>::new())?)
}
