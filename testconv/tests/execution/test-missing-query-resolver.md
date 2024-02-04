# test-missing-query-resolver

###### sdl error

#### server:

```graphql
schema {
  query: Query
}

type Query {
  user: [User]
  posts: [Post]!
}

type User {
  id: ID
  name: String
}

type Post {
  id: ID
}
```
