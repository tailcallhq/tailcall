# Test const with mustache

```graphql @server
schema {
  query: Query
}

type A {
  a: Int
  b: [Int] @modify(omit: true)
  bc: BC @const(data: {b: "{{value.b}}", c: "{{value.c}}", d: "{{value.d.e}}", f: "{{value.d}}", g: "{{value.g}}"})
  c: String @modify(omit: true)
  d: D @modify(omit: true)
  g: Boolean @modify(omit: true)
}

type BC {
  b: [Int]
  c: String
  d: Int
  f: D
  g: Boolean
}

type D {
  e: Int
}

type Query {
  a: A @const(data: {a: 0, b: [1, 2, 3], c: "test", d: {e: 1}, g: true})
}
```

```yml @assert
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: query { a { bc { b c d f {e}, g } } }
```
