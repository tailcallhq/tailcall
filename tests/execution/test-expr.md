---
identity: true
---

# test-expr

```graphql @schema
schema {
  query: Query
}

type Query {
  hello: String @expr(body: "Hello from server")
}
```
