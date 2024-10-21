---
error: true
---

# Cors invalid allowMethods

```graphql @config
schema
  @upstream(batch: {delay: 1, maxSize: 1000})
  @server(headers: {cors: {allowCredentials: true}}) {
  query: Query
}

type Query {
  val: Int @expr(body: 1)
}
```
