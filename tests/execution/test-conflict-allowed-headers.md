# test-conflict-allowed-headers

```graphql @server
schema @upstream(allowedHeaders: ["a", "b", "c"]) {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```

```graphql @server
schema @upstream(allowedHeaders: ["b", "c", "d"]) {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```
