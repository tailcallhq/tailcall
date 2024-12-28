---
error: true
---

# Test invalid add fields

```graphql @schema
schema @server(port: 8000) {
  query: Query
}

type Query {
  postuser: [PostUser] @http(url: "http://jsonplaceholder.typicode.com/posts")
}

type PostUser @addField(name: "username", path: "{{.value.user.username}}") {
  id: Int! @modify(name: "postId")
  title: String!
  userId: Int!
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/{{.value.userId}}")
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
