---
error: true
---

# Cors invalid exposeHeaders

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
  headers:
    cors:
      allowCredentials: true
      exposeHeaders: ["*"]
      allowMethods: [POST, OPTIONS]
upstream:
  batch: {delay: 1, maxSize: 1000}
```
