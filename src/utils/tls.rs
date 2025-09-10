use anyhow::Error;
use http_body_util::Empty;
use hyper::{Request, body::Bytes};

/// Builds an HTTP request with common headers for TLSNotary attestation
pub fn build_request(
    uri: &str,
    server_name: &str,
    extra_headers: &[(&str, &str)],
    description: &str,
    user_agent: &str,
) -> Result<Request<Empty<Bytes>>, Error> {
    println!("📡 Building request: {}", description);
    println!("   Endpoint: {}", uri);

    // Using "identity" instructs the Server not to use compression for its HTTP response.
    // TLSNotary tooling does not support compression.
    let request_builder = extra_headers.iter().fold(
        Request::builder()
            .uri(uri)
            .header("Host", server_name)
            .header("Accept", "*/*")
            .header("Accept-Encoding", "identity")
            .header("Connection", "close")
            .header("User-Agent", user_agent),
        |builder, (key, value)| builder.header(*key, *value),
    );

    Ok(request_builder.body(Empty::<Bytes>::new())?)
}
