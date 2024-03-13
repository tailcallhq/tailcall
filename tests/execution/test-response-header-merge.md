# test-response-header-value

```graphql @server
schema @server(headers: {custom: [{key: "a", value: "a"}]}) {
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

```graphql @server
schema @server(headers: {custom: [{key: "a", value: "b"}]}) {
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
