---
error: true
---

# Test invalid add fields

```graphql @config
schema @server(port: 8000) @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Query {
  postuser: [PostUser] @http(path: "/posts")
}

type PostUser @addField(name: "username", path: "{{.value.user.username}}") {
  id: Int! @modify(name: "postId")
  title: String!
  userId: Int!
  user: User @http(baseURL: "https://jsonplaceholder.typicode.com", path: "/users/{{.value.userId}}")
}

type User {
  id: Int!
  name: String!
  username: String!
  email: String!
  phone: String
  website: String
}
```
