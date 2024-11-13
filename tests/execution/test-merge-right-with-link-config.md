# test-merge-right-with-link-config

```graphql @file:stripe-types.graphql
type Foo {
  bar: String
}
```

```graphql @
schema @link(src: "config.yml", type: Config) @link(src: "stripe-types.graphql", type: Config) {
  query: Query
}

type Query {
  foo: Foo @expr(body: {bar: "foo"})
}
```

```yml @file:config.yml
schema: {}
upstream:
  allowedHeaders: ["Authorization"]
```
