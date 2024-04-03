---
check_identity: true
---

# test-const

```graphql @server
schema {
  query: Query
}

type Query {
  hello: String @const(data: "Hello from server")
}
```
