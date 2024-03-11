# undeclared-type-no-base-url

###### sdl error


```graphql @server
schema @server {
  query: Query
}

type Query {
  users: [User] @http(path: "/users")
}
```
