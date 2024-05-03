---
expect_validation_error: true
---

# test-multiple-resolvable-directives-on-field

```graphql @server
schema @upstream(baseURL: "https://jsonplaceholder.typicode.com") {
  query: Query
}

type Query {
  user1: User @http(path: "/users/1") @expr(body: {name: "John"})
  user2: User @http(path: "/users/2") @call(steps: [{query: "something"}])
}

type User {
  id: Int
  name: String
}
```
