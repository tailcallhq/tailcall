# test-const-error

###### sdl error

#### server:

```graphql
schema @server @upstream(baseURL: "https://jsonplaceholder.typicode.com") {
  query: Query
}

type User {
  name: String
  age: Int!
}

type Query {
  user: User @const(data: {name: "John"})
}
```
