---
error: true
---

# Cors invalid allowMethods

```yaml @config
upstream:
  batch:
    delay: 1
    maxSize: 1000
server:
  headers:
    cors:
      allowCredentials: true
```

```graphql @schema
schema {
  query: Query
}

type Query {
  val: Int @expr(body: 1)
}
```
