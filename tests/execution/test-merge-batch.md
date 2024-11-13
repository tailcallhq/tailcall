# test-merge-batch

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
  batch: {delay: 0, maxSize: 1000, headers: ["a", "b"]}
```

```graphql @config
schema {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
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
  batch: {delay: 5, maxSize: 100, headers: ["b", "c"]}
```
