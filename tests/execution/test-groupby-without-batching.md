# test-groupby-without-batching

###### sdl error

#### server:

```graphql
schema @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: true) {
  query: Query
}

type User {
  id: Int
  name: String
}

type Query {
  user(id: Int!): User @http(path: "/users", query: [{key: "id", value: "{{args.id}}"}], groupBy: ["id"])
}
```
