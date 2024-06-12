# Verify the correct CORS headers for requests from https://tailcall.run

```graphql @config
schema
  @upstream(baseURL: "http://example.com")
  @server(
    port: 8000
    headers: {
      cors: {allowHeaders: ["*"], allowMethods: ["GET", "POST", "OPTIONS"], allowOrigins: ["https://tailcall.run"]}
    }
  ) {
  query: Query
}

type Query {
  example: String @http(path: "/example")
}
```

```yml @test
# CORS test
- method: OPTIONS
  url: http://localhost:8000/graphql
  headers:
    Origin: https://tailcall.run
  expected:
    status: 200
    headers:
      access-control-allow-origin: https://tailcall.run
      access-control-allow-methods: GET, POST, OPTIONS
      access-control-allow-headers: "*"

# CORS test for GET request with Origin https://tailcall.run
- method: GET
  url: http://localhost:8000/graphql
  headers:
    Origin: https://tailcall.run
  expected:
    status: 200
    headers:
      access-control-allow-origin: https://tailcall.run

# CORS test for POST request with Origin https://tailcall.run
- method: POST
  url: http://localhost:8000/graphql
  headers:
    Origin: https://tailcall.run
    Content-Type: application/json
  body: '{"query": "{ example }"}'
  expected:
    status: 200
    headers:
      access-control-allow-origin: https://tailcall.run

# Additional CORS test for preflight request (OPTIONS) with different Origin
- method: OPTIONS
  url: http://localhost:8000/graphql
  headers:
    Origin: https://different-origin.com
  expected:
    status: 200
    headers:
      access-control-allow-origin: null

# Additional CORS test for GET request with different Origin
- method: GET
  url: http://localhost:8000/graphql
  headers:
    Origin: https://different-origin.com
  expected:
    status: 200
    headers:
      access-control-allow-origin: null

# Additional CORS test for POST request with different Origin
- method: POST
  url: http://localhost:8000/graphql
  headers:
    Origin: https://different-origin.com
    Content-Type: application/json
  body: '{"query": "{ example }"}'
  expected:
    status: 200
    headers:
      access-control-allow-origin: null

# Additional CORS test for OPTIONS request with custom headers
- method: OPTIONS
  url: http://localhost:8000/graphql
  headers:
    Origin: https://tailcall.run
    Access-Control-Request-Headers: X-Custom-Header
  expected:
    status: 200
    headers:
      access-control-allow-origin: https://tailcall.run
      access-control-allow-methods: GET, POST, OPTIONS
      access-control-allow-headers: "*"

# Additional CORS test for GET request with custom headers
- method: GET
  url: http://localhost:8000/graphql
  headers:
    Origin: https://tailcall.run
    X-Custom-Header: value
  expected:
    status: 200
    headers:
      access-control-allow-origin: https://tailcall.run
```
