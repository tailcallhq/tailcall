---
error: true
---

# test-http-with-add-field

```graphql @schema
schema @server {
  query: Query
}

type Query @addField(name: "name", path: ["post", "user", "name"]) {
  post: Post @http(url: "http://jsonplaceholder.typicode.com/posts/1")
}

type Post {
  id: Int
  title: String
  body: String
  userId: Int!
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/{{.value.userId}}")
}

type User {
  id: Int
  name: String
}
```
