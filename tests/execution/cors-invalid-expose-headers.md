---
error: true
---

# Cors invalid exposeHeaders

```graphql @config
schema
  @upstream(baseURL: "http://example.com", batch: {delay: 1, maxSize: 1000})
  @server(headers: {cors: {allowCredentials: true, exposeHeaders: ["*"], allowMethods: [POST, OPTIONS]}}) {
  query: Query
}

type Query {
  val: Int @expr(body: 1)
}
```
