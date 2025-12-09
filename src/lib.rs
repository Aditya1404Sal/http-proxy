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
                    streaming_handler::stream_handle(&prompt, response_out);
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

bindings::export!(Component with_types_in bindings);
