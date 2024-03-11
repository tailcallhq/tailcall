# test-all-blueprint-errors

###### sdl error


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
  bar: String @const @const(data: {name: "John"})
}
type Foo {
  a: String @const(data: "1")
  b: B
}
type B {
  a: String
}
```
