# test-response-header-value

```graphql @server
schema @server(headers: {custom: [{key: "a", value: "a"}]}) {
  query: Query
}

type Query {
  user: User @const(data: {name: "John"})
}

type User {
  age: Int
  name: String
}
```

```graphql @server
schema @server(headers: {custom: [{key: "a", value: "b"}]}) {
  query: Query
}

type Query {
  user: User @const(data: {name: "John"})
}

type User {
  age: Int
  name: String
}
```
