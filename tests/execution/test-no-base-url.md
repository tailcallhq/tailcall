# test-no-base-url

###### sdl error

#### server:

```graphql
schema {
  query: Query
}

type User {
  id: ID!
}

type Query {
  user: User @http(path: "/user/1")
}
```
