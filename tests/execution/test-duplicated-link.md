---
error: true
---

# test-duplicated-link

```graphql @file:jsonplaceholder.graphql
schema @server(port: 8000, hostname: "0.0.0.0") @upstream(httpCache: 42, batch: {delay: 100}) {
  query: Query
}

type Query {
  posts: [Post] @http(url: "http://jsonplaceholder.typicode.com/posts")
  users: [User] @http(url: "http://jsonplaceholder.typicode.com/users")
  user(id: Int!): User @http(url: "http://jsonplaceholder.typicode.com/users/{{.args.id}}")
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
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/{{.value.userId}}")
}
```

```graphql @schema
schema
  @link(type: Config, src: "jsonplaceholder.graphql", id: "placeholder")
  @link(type: Config, src: "jsonplaceholder.graphql", id: "placeholder1")
  @link(type: Config, src: "jsonplaceholder.graphql", id: "placeholder1")
  @link(type: Config, src: "jsonplaceholder.graphql", id: "placeholder2")
  @link(type: Config, src: "jsonplaceholder.graphql", id: "placeholder2") {
  query: Query
}

type Query {
  posts: [Post] @http(url: "http://jsonplaceholder.typicode.com/posts")
  user(id: Int!): User @http(url: "http://jsonplaceholder.typicode.com/users/{{.args.id}}")
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
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/{{.value.userId}}")
}
```
