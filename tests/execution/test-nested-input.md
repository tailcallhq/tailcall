# test-nested-input

```graphql @server
schema @server @upstream {
  query: Query
}

input A {
  b: B
}

input B {
  c: C
}

input C {
  d: D
}

input D {
  e: Int
}

type Query {
  a(a: A!): X @const(data: {a: "hello"})
}

type X {
  a: String
}
```
