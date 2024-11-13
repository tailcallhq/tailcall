# test-conflict-allowed-headers

```graphql @config
schema {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```

```yml @file:config.yml
schema: {}
upstream:
  allowedHeaders: ["a", "b", "c"]
```

```graphql @config
schema {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```

```yml @file:config.yml
schema: {}
upstream:
  allowedHeaders: ["b", "c", "d"]
```
