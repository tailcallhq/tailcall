---
error: true
---

# test-enum-empty

```graphql @schema
schema @server {
  query: Query
}

type Query {
  foo(val: String!): Foo @expr(body: "{{.args.val}}")
}

enum Foo {}
```
