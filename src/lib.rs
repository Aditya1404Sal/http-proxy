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
            streaming_handler::stream_handle(request, response_out);
        }

        _ => {
            eprintln!("[PROXY] No route matched, returning 405");
            method_not_allowed(response_out);
        }
    }
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