---
expect_validation_error: true
---

# test-experimental-headers-error

```graphql @server
schema @server(headers: {experimental: ["bar", "foo", "non-experimental", "tailcall"]}) {
  query: Query
}

type Query {
  hello: String @expr(body: "World!")
}
```
