---
expect_validation_error: true
---

# test-response-header-value

```graphql @server
schema @server(headers: {custom: [{key: "a", value: "a \n b"}]}) {
  query: Query
}

type User {
  name: String
  age: Int
}

type Query {
  user: User @const(data: {name: "John"})
}
```
