# HTTP Proxy for OpenAI

This is an HTTP component that wraps and routes requests to an OpenAI streaming component. It provides HTTP routing and proxies streaming responses from the OpenAI API.

## Prerequisites

- `cargo` 1.82
- [`wash`](https://wasmcloud.com/docs/installation) 0.36.1
- `wasmtime` >=25.0.0 (if running with wasmtime)

## Building

```bash
wash build
```

## Composing with OpenAI Component

This proxy component needs to be composed with the OpenAI component:

```bash
wac plug ./build/http_proxy.wasm --plug ../openai-component/build/openai_component.wasm -o final.wasm
```

## Running with Wasmtime

Set your OpenAI API key and run the composed component:

```bash
export OPENAI_API_KEY='api_key'
wasmtime serve -Scommon -Sinherit-env=y ./final.wasm
```

## Routes

- `POST /openai-proxy` - OpenAI proxy endpoint (delegates to OpenAI component)

## Running with wasmCloud

```shell
wash dev
```

## Architecture

This component acts as an HTTP proxy that:
- Receives HTTP requests with text prompts
- Delegates prompt to the OpenAI component via WIT bindings
- Uses the WASI HTTP interface for protocol-level request/response handling

## Adding Capabilities

To learn how to extend this example with additional capabilities, see the [Adding Capabilities](https://wasmcloud.com/docs/tour/adding-capabilities?lang=rust) section of the wasmCloud documentation.
