# test-merge-batch

```graphql @schema
schema {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```

```yml @config
schema: {}
upstream:
  batch: {delay: 0, maxSize: 1000, headers: ["a", "b"]}
```

```graphql @schema
schema {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```

```graphql @schema
schema {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```

```yml @config
schema: {}
upstream:
  batch: {delay: 5, maxSize: 100, headers: ["b", "c"]}
```
