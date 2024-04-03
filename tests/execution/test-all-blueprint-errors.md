---
expect_validation_error: true
---

# test-all-blueprint-errors

```graphql @server
schema @server {
  query: Query
  mutation: Mutation
}
type Mutation {
  a: String
}
type Query {
  foo(inp: B): Foo
  bar: String @const @expr(body: {name: "John"})
}
type Foo {
  a: String @expr(body: "1")
  b: B
}
type B {
  a: String
}
```
