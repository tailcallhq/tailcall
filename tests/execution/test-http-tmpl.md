---
identity: true
---

# test-http-tmpl

```graphql @schema
schema @server @upstream {
  query: Query
}

type Post {
  id: Int
  user: User @http(url: "http://jsonplaceholder.typicode.com/users", query: [{key: "id", value: "{{.value.userId}}"}])
  userId: Int!
}

type Query {
  posts: [Post] @http(url: "http://jsonplaceholder.typicode.com/posts")
}

type User {
  id: Int
  name: String
}
```
