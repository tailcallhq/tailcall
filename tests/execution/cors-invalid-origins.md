---
error: true
---

# Cors invalid allowOrigins

```graphql @schema
schema
  @upstream(batch: {delay: 1, maxSize: 1000})
  @server(headers: {cors: {allowCredentials: true, allowOrigins: ["*"], allowMethods: [POST, OPTIONS]}}) {
  query: Query
}

type Query {
  val: Int @expr(body: 1)
}
```
