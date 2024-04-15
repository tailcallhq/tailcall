---
expect_validation_error: true
---

# test-response-header-value

```graphql @server
schema @server(headers: {custom: [{key: "a", value: "a \n b"}]}) {
  query: Query
}

type Query {
  user: User @expr(body: {name: "John"})
}

type User {
  age: Int
  name: String
}
```
