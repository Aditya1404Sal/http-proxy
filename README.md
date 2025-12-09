# MCP Wrapper

This is a HTTP component that wraps and routes requests to an MCP (Model Context Protocol) component. It provides HTTP routing for health checks, actions, and MCP protocol handling.

## Prerequisites

- `cargo` 1.82
- [`wash`](https://wasmcloud.com/docs/installation) 0.36.1
- `wasmtime` >=25.0.0 (if running with wasmtime)

## Building

```bash
wash build
```

## Routes

- `GET /health` - Health check endpoint
- `POST /actions` - Actions endpoint (Betty Blocks integration - currently mocked)
- `POST /mcp` - MCP protocol endpoint (delegates to MCP component)

## Running with wasmCloud

```shell
wash dev
```

Test the endpoints - see [Testing.md](./Testing.md) for detailed testing instructions.

## Architecture

See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed architecture documentation.

This component acts as an HTTP router that:
- Handles basic HTTP routes (health, actions)
- Delegates MCP protocol requests to an imported MCP component via WIT bindings
- Uses the WASI HTTP interface for protocol-level request/response handling

## Adding Capabilities

To learn how to extend this example with additional capabilities, see the [Adding Capabilities](https://wasmcloud.com/docs/tour/adding-capabilities?lang=rust) section of the wasmCloud documentation.
