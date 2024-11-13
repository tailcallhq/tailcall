# test-conflict-allowed-headers

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
  allowedHeaders: ["a", "b", "c"]
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
  allowedHeaders: ["b", "c", "d"]
```
