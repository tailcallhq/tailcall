---
expect_validation_error: true
---

# test-http-with-add-field

```graphql @server
schema @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Post {
  body: String
  id: Int
  title: String
  user: User @http(path: "/users/{{value.userId}}")
  userId: Int!
}

type Query @addField(name: "name", path: ["post", "user", "name"]) {
  post: Post @http(path: "/posts/1")
}

type User {
  id: Int
  name: String
}
```
