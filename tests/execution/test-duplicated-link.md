# test-duplicated-link

###### sdl error

#### server:

```graphql
schema
  @link(type: Config, src: "../../examples/jsonplaceholder.graphql", id: "placeholder")
  @link(type: Config, src: "../../examples/jsonplaceholder.graphql", id: "placeholder1")
  @link(type: Config, src: "../../examples/jsonplaceholder.graphql", id: "placeholder1")
  @link(type: Config, src: "../../examples/jsonplaceholder.graphql", id: "placeholder2")
  @link(type: Config, src: "../../examples/jsonplaceholder.graphql", id: "placeholder2") {
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
