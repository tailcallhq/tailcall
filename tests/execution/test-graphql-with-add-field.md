---
error: true
---

# test-graphql-with-add-field

```graphql @schema
schema @server {
  query: Query
}

type Query @addField(name: "name", path: ["post", "user", "name"]) {
  post: Post @graphQL(url: "http://jsonplaceholder.typicode.com", name: "posts")
}

type Post {
  id: Int
  title: String
  body: String
  userId: Int!
  user: User @graphQL(url: "http://jsonplaceholder.typicode.com", name: "user")
}

type User {
  id: Int
  name: String
}
```
