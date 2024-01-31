# test-conflict-allowed-headers

#### server:

```graphql
schema @server @upstream(allowedHeaders: ["a", "b", "c"]) {
  query: Query
}

type Query {
  hello: String @const(data: "world")
}
```

#### server:

```graphql
schema @server @upstream(allowedHeaders: ["b", "c", "d"]) {
  query: Query
}

type Query {
  hello: String @const(data: "world")
}
```
