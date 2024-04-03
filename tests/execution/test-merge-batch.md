# test-merge-batch

```graphql @server
schema @upstream(batch: {delay: 0, headers: ["a", "b"], maxSize: 1000}) {
  query: Query
}

type Query {
  hello: String @const(data: "world")
}
```

```graphql @server
schema @upstream(batch: {delay: 5, headers: ["b", "c"], maxSize: 100}) {
  query: Query
}

type Query {
  hello: String @const(data: "world")
}
```

```graphql @server
schema {
  query: Query
}

type Query {
  hello: String @const(data: "world")
}
```
