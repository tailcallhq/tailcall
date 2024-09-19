---
error: true
---

# Cors invalid allowOrigins

```graphql @config
schema
  @upstream(baseURL: "http://example.com")
  @server(headers: {cors: {allowCredentials: true, allowOrigins: ["*"], allowMethods: [POST, OPTIONS]}}) {
  query: Query
}

type Query {
  val: Int @expr(body: 1)
}
```
