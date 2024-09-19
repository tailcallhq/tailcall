---
error: true
---

# Cors invalid allowHeaders

```graphql @config
schema
  @upstream(baseURL: "http://example.com")
  @server(headers: {cors: {allowCredentials: true, allowHeaders: ["*"], allowMethods: [POST, OPTIONS]}}) {
  query: Query
}

type Query {
  val: Int @expr(body: 1)
}
```
