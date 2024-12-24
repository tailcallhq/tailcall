# test-merge-right-with-link-config

```yaml @config
upstream:
  allowedHeaders: ["Authorization"]
links:
  - src: "stripe-types.graphql"
    type: Config
```

```graphql @file:stripe-types.graphql
type Foo {
  bar: String
}
```

```graphql @schema
schema {
  query: Query
}

type Query {
  foo: Foo @expr(body: {bar: "foo"})
}
```
