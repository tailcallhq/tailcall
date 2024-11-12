---
error: true
---

# Cors invalid allowOrigins

```graphql @config
schema @link(src: "config.yml", type: Config) {
  query: Query
}

type Query {
  val: Int @expr(body: 1)
}
```

```yml @file:config.yml
server:
  headers:
    cors:
      allowCredentials: true
      exposeHeaders: ["*"]
      allowMethods: [POST, OPTIONS]
upstream:
  batch: {delay: 1, maxSize: 1000}
```
