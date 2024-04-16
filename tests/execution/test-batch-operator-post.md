---
expect_validation_error: true
---

# test-batch-operator-post

```graphql @server
schema @upstream(baseURL: "http://localhost:3000", batch: {delay: 1, maxSize: 100}) {
  query: Query
}

type Query {
  user: User @http(batchKey: ["id"], method: "POST", path: "/posts/1")
}

type User {
  age: Int
  name: String
}
```
