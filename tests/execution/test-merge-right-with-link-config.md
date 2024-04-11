# test-merge-right-with-link-config

```graphql @file:stripe-types.graphql
type Foo {
  bar: String
}
```

```graphql @server
schema @upstream(allowedHeaders: ["Authorization"]) @link(src: "stripe-types.graphql", type: Config) {
  query: Query
}

type Query {
  foo: Foo @const(data: "foo")
}
```
