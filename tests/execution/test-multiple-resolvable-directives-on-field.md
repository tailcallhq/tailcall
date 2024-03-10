# test-multiple-resolvable-directives-on-field

###### sdl error

####
```graphql @server
schema @server @upstream(baseURL: "https://jsonplaceholder.typicode.com") {
  query: Query
}

type User {
  name: String
  id: Int
}

type Query {
  user1: User @const(data: {name: "John"}) @http(path: "/users/1")
  user2: User @http(path: "/users/2") @call(query: "something")
}
```
