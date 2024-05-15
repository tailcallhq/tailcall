# test-conflict-allowed-headers

```graphql @config
schema @server @upstream(allowedHeaders: ["a", "b", "c"]) {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```

```graphql @config
schema @server @upstream(allowedHeaders: ["b", "c", "d"]) {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```
