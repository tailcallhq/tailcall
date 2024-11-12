---
identity: true
---

# test-nested-value

```graphql @config
schema {
  query: Query
}

type Post {
  id: Int
  user: User! @http(url: "http://jsonplaceholder.typicode.com/users", query: [{key: "id", value: "{{.value.user.id}}"}])
}

type Query {
  posts: [Post] @http(url: "http://jsonplaceholder.typicode.com/posts")
}

type User {
  id: Int!
  name: String
}
```
