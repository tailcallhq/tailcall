---
expect_validation_error: true
---

# Cors invalid allowOrigins

```graphql @server
schema @server(headers: {cors: {allowCredentials: true, allowMethods: ["POST", "OPTIONS"], allowOrigins: ["*"], vary: ["origin", "access-control-request-method", "access-control-request-headers"]}}) @upstream(baseURL: "http://example.com", batch: {delay: 1, headers: [], maxSize: 1000}) {
  query: Query
}

type Query {
  val: Int @expr(body: 1)
}
```
