---
error: true
---

# test-groupby-without-batching

```graphql @schema
schema @upstream(httpCache: 42) {
  query: Query
}

type User {
  id: Int
  name: String
}

type Query {
  user(id: Int!): User
    @http(
      url: "http://jsonplaceholder.typicode.com/users"
      query: [{key: "id", value: "{{.args.id}}"}]
      batchKey: ["id"]
    )
}
```
