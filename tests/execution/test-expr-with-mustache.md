# Test expr with mustache

```graphql @schema
schema {
  query: Query
}

type Query {
  a: A @expr(body: {a: 0, b: [1, 2, 3], c: "test", d: {e: 1}, g: true})
}

type A {
  a: Int
  b: [Int] @modify(omit: true)
  c: String @modify(omit: true)
  g: Boolean @modify(omit: true)
  d: D @modify(omit: true)
  bc: BC @expr(body: {b: "{{.value.b}}", c: "{{.value.c}}", d: "{{.value.d.e}}", f: "{{.value.d}}", g: "{{.value.g}}"})
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
```

```yml @test
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: query { a { bc { b c d f {e}, g } } }
```
