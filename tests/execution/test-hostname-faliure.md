---
error: true
---

# test-hostname-faliure

```graphql @schema
schema @server(hostname: "abc") {
  query: Query
}

type User {
  id: Int
  name: String
}

type Query {
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/1")
}
```
