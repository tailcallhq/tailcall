---
error: true
---

# test-invalid-query-in-http

```graphql @schema
schema @server(vars: [{key: "id", value: "1"}]) {
  query: Query
}

type User {
  name: String
  id: Int
}

type Query {
  user: [User] @http(url: "http://jsonplaceholder.typicode.com/users", query: {key: "id", value: "{{.vars.id}}"})
}
```
