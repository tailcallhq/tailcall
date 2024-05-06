---
expect_validation_error: true
---

# test-enum-empty

```graphql @server
schema @server @upstream(baseURL: "http://localhost:8080") {
  query: Query
}

enum Foo {
}

type Query {
  foo(val: String!): Foo @expr(body: "{{.args.val}}")
}
```
