# undeclared-type

###### sdl error

```graphql @server
schema @server {
  query: Query
}

type Query {
  users: [User] @http(baseURL: "https://jsonplaceholder.typicode.com", path: "/users")
}
```
