# test-server-base-types

```graphql @config
schema {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```

```graphql @config
schema  @upstream(proxy: {url: "http://localhost:8000"}) {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```
