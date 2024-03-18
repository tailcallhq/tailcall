# test-batch-operator-post

---
expect_validation_error: true
---

```graphql @server
schema @server @upstream(baseURL: "http://localhost:3000", batch: {delay: 1}) {
  query: Query
}

type User {
  name: String
  age: Int
}

type Query {
  user: User @http(path: "/posts/1", method: "POST", batchKey: ["id"])
}
```
