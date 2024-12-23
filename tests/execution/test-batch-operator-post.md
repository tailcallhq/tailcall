---
error: true
---

# test-batch-operator-post

```graphql @schema
schema @server @upstream(batch: {delay: 1}) {
  query: Query
}

type User {
  name: String
  age: Int
}

type Query {
  user: User @http(url: "http://localhost:3000/posts/1", method: "POST", batchKey: ["id"])
}
```
