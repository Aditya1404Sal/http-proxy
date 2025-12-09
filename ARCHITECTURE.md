# Architecture Decision: Raw WASI HTTP Bindings

## Context

This component started as a simple HTTP wrapper using `wasmcloud_component::http::Server`, which provides a convenient higher-level API for building HTTP servers. However, to integrate MCP (Model Context Protocol) component delegation, we needed to migrate to raw WASI HTTP bindings.

## Original Implementation

The original component used `wasmcloud_component::http`:

```rust
use wasmcloud_component::http;

impl http::Server for Component {
    fn handle(
        request: http::IncomingRequest,
    ) -> http::Result<http::Response<impl http::OutgoingBody>> {
        // Business logic here
    }
}

http::export!(Component);
```

This provided:
- High-level `Request` and `Response` types from the `http` crate
- Automatic conversion between WASI types and Rust types
- Convenient response building with `http::Response::builder()`
- Implicit response handling through return values

## The Problem

The imported MCP handler has this signature:

```wit
interface mcp-handler {
    use wasi:http/types@0.2.2.{incoming-request, response-outparam};
    
    mcp-handle: func(request: incoming-request, response: response-outparam);
}
```

This function expects:
1. **Raw WASI `incoming-request`** - not the wrapped `http::Request` type
2. **Raw WASI `response-outparam`** - a callback-style response parameter, not a return value

### Type Incompatibility

The `wasmcloud_component::http` wrapper uses different types:
- `http::IncomingRequest` = `Request<IncomingBody>` (from `http` crate)
- `http::Response<impl OutgoingBody>` (returned value)

The MCP handler needs:
- `wasi::http::types::IncomingRequest` (raw WASI resource)
- `wasi::http::types::ResponseOutparam` (passed as parameter)

### The Core Issue

When using `http::Server`, the trait signature returns a `Response`, which is then converted and sent by the framework. But `mcp_handle` needs to:
1. Receive the raw WASI request resource (which gets consumed/moved)
2. Send the response directly through the `ResponseOutparam` callback

There's no way to "extract" the raw WASI types from the wrapped types, or to delegate response handling to `mcp_handle` while still returning a response from the `http::Server` trait.

## The Solution

Implement the raw WASI HTTP interface directly:

```rust
use bindings::wasi::http::types::{
    IncomingRequest, ResponseOutparam, OutgoingResponse, Fields, OutgoingBody, Method
};

use crate::bindings::exports::wasi::http::incoming_handler::Guest;

impl Guest for Component {
    fn handle(request: IncomingRequest, response_out: ResponseOutparam) {
        // Route and handle requests
        // Can pass request and response_out directly to mcp_handle
    }
}

bindings::export!(Component with_types_in bindings);
```

### Benefits

1. **Direct Type Compatibility**: We work with the same `IncomingRequest` and `ResponseOutparam` types that `mcp_handle` expects
2. **Zero-Copy Delegation**: Can pass the request directly to `mcp_handle` without conversion
3. **Response Control**: Full control over when and how responses are sent
4. **Proper Routing**: Can inspect the request before deciding whether to delegate to MCP or handle locally

### Trade-offs

1. **More Verbose**: Must manually construct `OutgoingResponse` and write to streams
2. **Manual Conversions**: Need to manually work with WASI types (e.g., `Method::Get` instead of `http::Method::GET`)
3. **Lower-Level API**: No convenient `Response::builder()` or automatic body handling

## Implementation Pattern

We use a two-layer architecture:

1. **`inner_handle(&IncomingRequest) -> Result<HandleResult, Error>`**
   - Business logic and routing
   - Returns either a response or a delegation marker
   - Takes request by reference to avoid premature consumption

2. **`Guest::handle(IncomingRequest, ResponseOutparam)`**
   - Protocol layer
   - Checks the result from `inner_handle`
   - Either sends the response or delegates to `mcp_handle`
   - Handles errors by converting them to HTTP error responses

This separates concerns while maintaining compatibility with both normal HTTP handling and MCP delegation.

## Conclusion

Using raw WASI HTTP bindings is **necessary** for this use case because:
- The MCP handler interface is defined in terms of raw WASI types
- We need to delegate the entire request/response cycle to the MCP component
- The `wasmcloud_component::http` wrapper abstracts away the types we need to pass through

While it requires more manual work, it's the only way to achieve proper interoperability with WIT-defined component imports that use WASI HTTP types directly.
