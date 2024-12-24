---
error: true
---

```yaml @config
upstream:
  httpCache: 42
  batch:
    delay: 1
```

# batching validation

```graphql @schema
schema {
  query: Query
}

type User {
  id: Int
  name: String
}

type Post {
  id: Int
  title: String
  body: String
}

type Query {
  user(id: Int!): User
    @http(
      url: "http://jsonplaceholder.typicode.com/users"
      method: POST
      body: {uId: "{{.args.id}}", userId: "{{.args.id}}"}
      batchKey: ["id"]
    )
  posts: [Post] @http(url: "http://jsonplaceholder.typicode.com/posts", batchKey: ["id"])
  userWithId(id: Int!): User
    @http(
      url: "http://jsonplaceholder.typicode.com/users"
      method: POST
      body: {uId: "uId", userId: "userId"}
      batchKey: ["id"]
    )
  userWithIdTest(id: Int!): User
    @http(
      url: "http://jsonplaceholder.typicode.com/users"
      method: PUT
      body: {uId: "uId", userId: "userId"}
      batchKey: ["id"]
    )
}
```
