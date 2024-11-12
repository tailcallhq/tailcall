# test-conflict-allowed-headers

```graphql @config
schema @upstream(allowedHeaders: ["a", "b", "c"]) {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```

```graphql @config
schema @upstream(allowedHeaders: ["b", "c", "d"]) {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```
