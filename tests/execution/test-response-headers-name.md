# test-response-headers-name

###### sdl error

```graphql @server
schema @server(headers: { custom: [{key: "🤣", value: "a"}]}) {
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
