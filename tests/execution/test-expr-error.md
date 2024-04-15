---
expect_validation_error: true
---

# test-expr-error

```graphql @server
schema @upstream(baseURL: "https://jsonplaceholder.typicode.com") {
  query: Query
}

type Query {
  user: User @expr(body: {name: "John"})
}

type User {
  age: Int!
  name: String
}
```
