# test-lack-resolver

###### sdl error


```graphql @server
schema @server(port: 8000) @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Query {
  posts: InPost
}

type InPost {
  get: [Post]
}

type Post {
  id: Int!
  userId: Int!
  title: String!
  body: String!
  user: User @http(path: "/users/1")
}

type User {
  name: String
  id: Int
}
```
