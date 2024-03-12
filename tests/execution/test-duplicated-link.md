# test-duplicated-link

###### sdl error

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
  @link(type: Config, src: "jsonplaceholder.graphql", id: "placeholder")
  @link(type: Config, src: "jsonplaceholder.graphql", id: "placeholder1")
  @link(type: Config, src: "jsonplaceholder.graphql", id: "placeholder1")
  @link(type: Config, src: "jsonplaceholder.graphql", id: "placeholder2")
  @link(type: Config, src: "jsonplaceholder.graphql", id: "placeholder2") {
  query: Query
}

type Query {
  posts: [Post] @http(path: "/posts")
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
