mod bindings {
    wit_bindgen::generate!({
        generate_all,
    });
}

use bindings::{
    exports::wasi::http::incoming_handler::Guest,
    wasi::http::types::{
        Fields, IncomingRequest, Method, OutgoingBody, OutgoingResponse, ResponseOutparam,
    },
    wasi::io::streams::StreamError,
    wasmcloud::ai::streaming_handler,
};

struct Component;

impl Guest for Component {
    fn handle(request: IncomingRequest, response_out: ResponseOutparam) {
        handle_request(request, response_out);
    }
}

bindings::export!(Component with_types_in bindings);

fn handle_request(request: IncomingRequest, response_out: ResponseOutparam) {
    let headers = request.headers().entries();

    eprintln!("[PROXY] Received request");
    eprintln!("[PROXY] Method: {:?}", request.method());
    eprintln!("[PROXY] Path: {:?}", request.path_with_query());
    eprintln!("[PROXY] Authority: {:?}", request.authority());
    eprintln!("[PROXY] Headers: {:?}", headers);

    match (request.method(), request.path_with_query().as_deref()) {
        (Method::Post, Some("/openai-proxy")) => {
            eprintln!("[PROXY] Matched /openai-proxy route, delegating to streaming-handler");

            // Extract prompt from request body
            match extract_prompt_from_request(&request) {
                Ok(prompt) => {
                    eprintln!("[PROXY] Extracted prompt: {}", prompt);

                    // Call the streaming handler and get the complete response
                    let response_text = streaming_handler::prompt_handle(&prompt);

                    eprintln!(
                        "[PROXY] Got response from streaming handler: {} bytes",
                        response_text.len()
                    );

                    // Send the response back to the client
                    send_text_response(response_out, 200, response_text);
                }
                Err(e) => {
                    eprintln!("[PROXY] Failed to extract prompt: {}", e);
                    bad_request(response_out);
                }
            }
        }

        _ => {
            eprintln!("[PROXY] No route matched, returning 405");
            method_not_allowed(response_out);
        }
    }
}

fn extract_prompt_from_request(request: &IncomingRequest) -> Result<String, String> {
    let body = request
        .consume()
        .map_err(|_| "Failed to consume request body")?;

    let input_stream = body.stream().map_err(|_| "Failed to get input stream")?;

    let mut prompt_bytes = Vec::new();
    loop {
        match input_stream.blocking_read(8192) {
            Ok(chunk) => {
                if chunk.is_empty() {
                    break;
                }
                prompt_bytes.extend_from_slice(&chunk);
            }
            Err(StreamError::Closed) => break,
            Err(e) => return Err(format!("Stream read error: {:?}", e)),
        }
    }

    String::from_utf8(prompt_bytes).map_err(|e| format!("Invalid UTF-8: {}", e))
}

fn send_text_response(response_out: ResponseOutparam, status: u16, text: String) {
    let headers = Fields::new();
    headers
        .append(&"content-type".to_string(), &b"text/plain".to_vec())
        .expect("failed to append content-type header");

    let response = OutgoingResponse::new(headers);
    response
        .set_status_code(status)
        .expect("setting status code");

    let body = response.body().expect("response should be writable");

    ResponseOutparam::set(response_out, Ok(response));

    let output_stream = body.write().expect("body should be writable");
    
    // Write in chunks of 4096 bytes (the max for blocking_write_and_flush)
    let bytes = text.as_bytes();
    const CHUNK_SIZE: usize = 4096;
    
    for chunk in bytes.chunks(CHUNK_SIZE) {
        output_stream
            .blocking_write_and_flush(chunk)
            .expect("failed to write response body chunk");
    }

    drop(output_stream);
    OutgoingBody::finish(body, None).expect("outgoing-body.finish");
}

fn bad_request(response_out: ResponseOutparam) {
    respond(400, response_out)
}

fn method_not_allowed(response_out: ResponseOutparam) {
    respond(405, response_out)
}

fn respond(status: u16, response_out: ResponseOutparam) {
    let response = OutgoingResponse::new(Fields::new());
    response
        .set_status_code(status)
        .expect("setting status code");

    let body = response.body().expect("response should be writable");

    ResponseOutparam::set(response_out, Ok(response));

    OutgoingBody::finish(body, None).expect("outgoing-body.finish");
}