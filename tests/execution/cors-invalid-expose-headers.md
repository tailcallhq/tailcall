---
error: true
---

# Cors invalid exposeHeaders

```graphql @config
schema {
  query: Query
}

type Query {
  val: Int @expr(body: 1)
}
```

```yml @file:config.yml
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
