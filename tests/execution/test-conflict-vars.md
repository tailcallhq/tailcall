# test-conflict-vars

```graphql @server
schema @server(vars: [{key: "a", value: "b"}, {key: "c", value: "d"}]) {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```

```graphql @server
schema @server(vars: [{key: "a", value: "b"}, {key: "p", value: "q"}]) {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```
