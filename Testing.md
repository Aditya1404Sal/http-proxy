# Testing the MCP Wrapper Component

This guide shows you how to test the MCP wrapper component endpoints after deployment.

## Prerequisites

- Running wash host
- Kubernetes cluster with the runtime operator
- MCP wrapper component deployed (image: `ghcr.io/aditya1404sal/mcp-wrapper:latest`)

## Deployment

Apply the MCP wrapper workload:

```bash
kubectl apply -f mcp.yaml
```

Verify the workload is ready:

```bash
kubectl get workload
# Should show: mcp-wrapper-xxx   READY: True
```

## Testing Endpoints

The component exposes three endpoints at `http://mcp.localhost.direct:8000`:

### 1. Health Check Endpoint

**GET /health**

```bash
curl -i -X GET http://mcp.localhost.direct:8000/health
```

Expected response:
```
HTTP/1.1 200 OK
transfer-encoding: chunked
date: Fri, 05 Dec 2025 10:02:37 GMT

healthy
```

---

### 2. Actions Endpoint

**POST /actions**

Test the actions endpoint with a sample action:

```bash
curl -i -X POST http://mcp.localhost.direct:8000/actions \
  -H "Content-Type: application/json" \
  -d '{
    "action_id": "test-action",
    "payload": {
      "input": "test input data"
    }
  }'
```

Expected response:
```
HTTP/1.1 200 OK
transfer-encoding: chunked

Action test-action would be called with payload: test input data
```

---

### 3. MCP Endpoint

**POST /mcp**

The MCP endpoint supports JSON-RPC 2.0 protocol.

#### Initialize Request

```bash
curl -i -X POST http://mcp.localhost.direct:8000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "capabilities": {}
    },
    "id": 1
  }'
```

Expected response:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "result": {
      "capabilities": {},
      "protocolVersion": "2024-11-05",
      "serverInfo": {
        "name": "rust-mcp-server",
        "version": "0.1.0"
      }
    }
  },
  "id": 1
}
```

#### Tools List Request

```bash
curl -i -X POST http://mcp.localhost.direct:8000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/list",
    "params": {},
    "id": 2
  }'
```

Expected response:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "result": {
      "tools": [{}]
    }
  },
  "id": 2
}
```

---


## Using the Component in Your Deployment

You can use this public component in your own workload:

```yaml
apiVersion: runtime.wasmcloud.dev/v1alpha1
kind: WorkloadDeployment
metadata:
  name: mcp-wrapper
spec:
  replicas: 1
  template:
    spec:
      hostInterfaces:
        - namespace: wasi
          package: http
          interfaces:
            - incoming-handler
          config:
            # public dns alias for 127.0.0.1
            host: mcp.localhost.direct:8000
        - namespace: wasi
          package: logging
          version: 0.1.0-draft
          interfaces:
            - logging
      components:
        - name: mcp-new
          image: ghcr.io/aditya1404sal/mcp-wrapper:latest
```