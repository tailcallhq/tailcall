# test-merge-right-with-link-config

```graphql @file:stripe-types.graphql
type Foo {
  bar: String
}
```

```graphql @
schema {
  query: Query
}

type Query {
  foo: Foo @expr(body: {bar: "foo"})
}
```

```yml @config
schema: {}
upstream:
  allowedHeaders: ["Authorization"]
links:
  - src: "stripe-types.graphql"
    type: Config
```
