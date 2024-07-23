---
error: true
---

# test-groupby-without-batching

```graphql @config
schema @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: 42) {
  query: Query
}

type User {
  id: Int
  name: String
}

type Query {
  user(id: Int!): User @http(path: "/users", query: [{key: "id", value: "{{.args.id}}"}], batchKey: ["id"])
}
```
