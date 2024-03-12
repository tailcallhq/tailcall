# test-no-base-url

###### sdl error

```graphql @server
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
