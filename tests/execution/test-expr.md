---
identity: true
---

# test-expr

```graphql @schema
schema @server @upstream {
  query: Query
}

type Query {
  hello: String @expr(body: "Hello from server")
}
```
