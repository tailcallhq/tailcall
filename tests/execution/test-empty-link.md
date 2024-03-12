# test-empty-link

###### sdl error

```graphql @server
schema @upstream(baseURL: "https://jsonplaceholder.typicode.com") @link(type: Config, src: "") @link(type: Config) {
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
