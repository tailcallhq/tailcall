---
expect_validation_error: true
---

# test-empty-link

```graphql @server
schema @upstream(baseURL: "https://jsonplaceholder.typicode.com") @link(type: Config) @link(type: Config) {
  query: Query
}

type Post {
  body: String!
  id: Int!
  title: String!
  user: User @http(path: "/users/{{value.userId}}")
  userId: Int!
}

type Query {
  posts: [Post] @http(path: "/posts")
  user(id: Int!): User @http(path: "/users/{{args.id}}")
}

type User {
  email: String!
  id: Int!
  name: String!
  phone: String
  username: String!
  website: String
}
```
