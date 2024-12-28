---
error: true
---

# test-http-with-inline

```graphql @schema
schema @server {
  query: Query
}

type Query @addField(name: "username", path: ["post", "user", "name"]) {
  post: Post
    @http(url: "http://jsonplaceholder.typicode.com/posts/1")
    @http(url: "http://jsonplaceholder.typicode.com/users/{{.value.userId}}")
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
