---
expect_validation_error: true
---

# test-hostname-faliure

```graphql @server
schema @server(hostname: "abc") {
  query: Query
}

type Query {
  user: User @http(baseURL: "http://jsonplaceholder.typicode.com", path: "/users/1")
}

type User {
  id: Int
  name: String
}
```
