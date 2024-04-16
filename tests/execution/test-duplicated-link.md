---
expect_validation_error: true
---

# test-duplicated-link

```graphql @file:jsonplaceholder.graphql
schema
  @server(port: 8000, graphiql: true, hostname: "0.0.0.0")
  @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: true, batch: {delay: 100}) {
  query: Query
}

type Query {
  posts: [Post] @http(path: "/posts")
  users: [User] @http(path: "/users")
  user(id: Int!): User @http(path: "/users/{{args.id}}")
}

type User {
  id: Int!
  name: String!
  username: String!
  email: String!
  phone: String
  website: String
}

type Post {
  id: Int!
  userId: Int!
  title: String!
  body: String!
  user: User @http(path: "/users/{{value.userId}}")
}
```

```graphql @server
schema
  @link(id: "placeholder", src: "jsonplaceholder.graphql", type: Config)
  @link(id: "placeholder1", src: "jsonplaceholder.graphql", type: Config)
  @link(id: "placeholder1", src: "jsonplaceholder.graphql", type: Config)
  @link(id: "placeholder2", src: "jsonplaceholder.graphql", type: Config)
  @link(id: "placeholder2", src: "jsonplaceholder.graphql", type: Config) {
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
