# test-conflict-vars

```graphql @server
schema @server(vars: [{key: "a", value: "b"}, {key: "c", value: "d"}]) @upstream {
  query: Query
}

type Query {
  hello: String @const(data: "world")
}
```

```graphql @server
schema @server(vars: [{key: "a", value: "b"}, {key: "p", value: "q"}]) @upstream {
  query: Query
}

type Query {
  hello: String @const(data: "world")
}
```
