---
expect_validation_error: true
---

# Cors invalid exposeHeaders

```graphql @server
schema @server(headers: {cors: {allowCredentials: true, allowMethods: ["POST", "OPTIONS"], exposeHeaders: ["*"], vary: ["origin", "access-control-request-method", "access-control-request-headers"]}}) @upstream(baseURL: "http://example.com", batch: {delay: 1, maxSize: 1000}) {
  query: Query
}

type Query {
  val: Int @expr(body: 1)
}
```
