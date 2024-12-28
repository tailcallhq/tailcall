---
error: true
---

# test-expr-with-inline

```graphql @schema
schema @server {
  query: Query
}

type Query @addField(name: "username", path: ["post", "user", "name"]) {
  post: Post @http(url: "http://jsonplaceholder.typicode.com/posts/1")
}

type Post {
  id: Int
  title: String
  body: String
  userId: Int
  user: User @expr(body: {id: 1, name: "user1"})
}

type User {
  id: Int
  name: String
}
```
