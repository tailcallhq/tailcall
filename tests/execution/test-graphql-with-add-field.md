---
error: true
---

# test-graphql-with-add-field

```graphql @config
schema @server @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Query @addField(name: "name", path: ["post", "user", "name"]) {
  post: Post @graphQL(name: "posts")
}

type Post {
  id: Int
  title: String
  body: String
  userId: Int!
  user: User @graphQL(name: "user")
}

type User {
  id: Int
  name: String
}
```
