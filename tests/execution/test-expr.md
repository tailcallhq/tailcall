---
identity: true
---

# test-expr

```graphql @server
schema @server @upstream {
  query: Query
}

type Query {
  hello: String @expr(body: "Hello from server")
}
```
