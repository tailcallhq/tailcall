---
identity: true
---

# test-expr

```graphql @config
schema {
  query: Query
}

type Query {
  hello: String @expr(body: "Hello from server")
}
```
