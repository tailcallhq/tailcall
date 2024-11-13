---
error: true
---

# Cors invalid allowMethods

```graphql @schema
schema {
  query: Query
}

type Query {
  val: Int @expr(body: 1)
}
```

```yml @config
schema: {}
server:
  headers: {cors: {allowCredentials: true}}
upstream:
  batch: {delay: 1, maxSize: 1000}
```
