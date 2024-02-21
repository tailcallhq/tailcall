# Test const with mustache

#### server:

```graphql
schema {
  query: Query
}

type Query {
  a: A @const(data: {a: 0, b: [1, 2, 3], c: "test"})
}

type A {
  a: Int
  b: [Int] @modify(omit: true)
  c: String @modify(omit: true)
  bc: BC @const(data: {b: "{{value.b}}", c: "{{value.c}}"})
}

type BC {
  b: [Int]
  c: String
}

```

#### assert:

```yml
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: query { a { bc { b c } } }
```
