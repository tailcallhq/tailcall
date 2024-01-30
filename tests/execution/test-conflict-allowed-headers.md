# test-conflict-allowed-headers.graphql

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

#### merged:

```graphql
schema @server @upstream(allowedHeaders: ["a", "b", "c", "d"]) {
  query: Query
}

type Query {
  hello: String @const(data: "world")
}
```
