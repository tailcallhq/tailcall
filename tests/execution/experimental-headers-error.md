---
error: true
---

# test-experimental-headers-error

```graphql @config
schema {
  query: Query
}

type Query {
  hello: String @expr(body: "World!")
}
```

```yml @file:config.yml
schema: {}
server:
  headers: {experimental: ["non-experimental", "foo", "bar", "tailcall"]}
```
