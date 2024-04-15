---
expect_validation_error: true
---

# test-expr-with-add-field

```graphql @server
schema @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Post {
  body: String
  id: Int
  title: String
  user: User @expr(body: {id: 1, name: "user1"})
  userId: Int
}

type Query @addField(name: "name", path: ["post", "user", "name"]) {
  post: Post @http(path: "/posts/1")
}

type User {
  id: Int
  name: String
}
```
