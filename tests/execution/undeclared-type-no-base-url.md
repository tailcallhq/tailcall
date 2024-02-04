# undeclared-type-no-base-url

###### sdl error

#### server:

```graphql
schema @server {
  query: Query
}

type Query {
  users: [User] @http(path: "/users")
}
```
