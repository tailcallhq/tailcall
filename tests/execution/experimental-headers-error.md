---
error: true
---

# test-experimental-headers-error

```graphql @schema
schema {
  query: Query
}

type Query {
  hello: String @expr(body: "World!")
}
```

```yml @config
schema: {}
server:
  headers: {experimental: ["non-experimental", "foo", "bar", "tailcall"]}
```
