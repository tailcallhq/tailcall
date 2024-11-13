# test-merge-query

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
  hi: String @expr(body: "world")
}
```

```yml @file:config.yml
schema: {}
upstream:
  proxy: {url: "http://localhost:8000"}
```
