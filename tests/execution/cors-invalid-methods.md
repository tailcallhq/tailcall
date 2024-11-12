---
error: true
---

# Cors invalid allowMethods

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
  headers: {cors: {allowCredentials: true}}
upstream:
  batch: {delay: 1, maxSize: 1000}
```
