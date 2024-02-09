# undeclared-type

###### sdl error

#### server:

```graphql
schema @server {
  query: Query
}

type Query {
  users: [User] @http(baseURL: "https://jsonplaceholder.typicode.com", path: "/users")
}
```
