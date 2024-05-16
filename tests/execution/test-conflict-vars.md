# test-conflict-vars

```graphql @config
schema @server(vars: [{key: "a", value: "b"}, {key: "c", value: "d"}]) @upstream {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```

```graphql @config
schema @server(vars: [{key: "a", value: "b"}, {key: "p", value: "q"}]) @upstream {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```
