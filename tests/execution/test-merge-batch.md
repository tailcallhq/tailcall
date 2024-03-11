# test-merge-batch


```graphql @server
schema @server @upstream(batch: {delay: 0, maxSize: 1000, headers: ["a", "b"]}) {
  query: Query
}

type Query {
  hello: String @const(data: "world")
}
```


```graphql @server
schema @server @upstream(batch: {delay: 5, maxSize: 100, headers: ["b", "c"]}) {
  query: Query
}

type Query {
  hello: String @const(data: "world")
}
```


```graphql @server
schema @server @upstream {
  query: Query
}

type Query {
  hello: String @const(data: "world")
}
```
