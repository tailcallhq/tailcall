---
expect_validation_error: true
---

# test-groupby-without-batching

```graphql @server
schema @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: true) {
  query: Query
}

type Query {
  user(id: Int!): User @http(batchKey: ["id"], path: "/users", query: [{key: "id", value: "{{args.id}}"}])
}

type User {
  id: Int
  name: String
}
```
