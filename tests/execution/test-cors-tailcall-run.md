# Verify the correct CORS headers for requests from https://tailcall.run

```graphql @config
schema
  @upstream(baseURL: "http://example.com", batch: {delay: 1, maxSize: 1000})
  @server(
    port: 8000,
    headers: {
      cors: {
        allowHeaders: ["*"]
        allowMethods: ["GET", "POST", "OPTIONS"]
        allowOrigins: ["https://tailcall.run"]
      }
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
```
