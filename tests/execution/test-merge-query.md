# test-merge-query

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
  hi: String @expr(body: "world")
}
```

```yml @config
schema: {}
upstream:
  proxy: {url: "http://localhost:8000"}
```
