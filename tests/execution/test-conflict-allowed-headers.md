# test-conflict-allowed-headers

####

```graphql @server
schema @server @upstream(allowedHeaders: ["a", "b", "c"]) {
  query: Query
}

type Query {
  hello: String @const(data: "world")
}
```

####

```graphql @server
schema @server @upstream(allowedHeaders: ["b", "c", "d"]) {
  query: Query
}

type Query {
  hello: String @const(data: "world")
}
```
