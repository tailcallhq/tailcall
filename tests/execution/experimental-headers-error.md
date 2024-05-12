---
error: true
---

# test-experimental-headers-error

```graphql @config
schema @server(headers: {experimental: ["non-experimental", "foo", "bar", "tailcall"]}) {
  query: Query
}

type Query {
  hello: String @expr(body: "World!")
}
```
