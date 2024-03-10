# test-response-header-value

###### sdl error

####
```graphql @server
schema @server(responseHeaders: [{key: "a", value: "a \n b"}]) {
  query: Query
}

type User {
  name: String
  age: Int
}

type Query {
  user: User @const(data: {name: "John"})
}
```
