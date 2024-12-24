---
error: true
---

# Cors invalid allowOrigins

```yaml @config
upstream:
  batch:
    delay: 1
    maxSize: 1000
server:
  headers:
    cors:
      allowCredentials: true
      allowOrigins: ["*"]
      allowMethods: [POST, OPTIONS]
```

```graphql @schema
schema {
  query: Query
}

type Query {
  val: Int @expr(body: 1)
}
```
