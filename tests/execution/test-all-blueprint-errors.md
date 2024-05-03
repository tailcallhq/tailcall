---
expect_validation_error: true
---

# test-all-blueprint-errors

```graphql @server
schema {
  query: Query
  mutation: Mutation
}

input B {
  a: String
}

type Foo {
  a: String @expr(body: "1")
  b: B
}

type Mutation {
  a: String
}

type Query {
  bar: String @expr(body: {name: "John"})
  foo(inp: B): Foo
}
```
