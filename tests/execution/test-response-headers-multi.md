# test-response-headers-multi

###### sdl error

```graphql @server
schema @server(responseHeaders: [{key: "a b", value: "a \n b"}, {key: "a c", value: "a \n b"}]) {
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
