# test-conflict-allowed-headers

```graphql @config
schema @link(src: "config-a.yml", type: Config) {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```

```yml @file:config-a.yml
schema: {}
upstream:
  allowedHeaders: ["a", "b", "c"]
```

```graphql @config
schema @link(src: "config-b.yml", type: Config) {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```

```yml @file:config-b.yml
schema: {}
upstream:
  allowedHeaders: ["b", "c", "d"]
```
