# Test const with mustache

#### server:

```graphql
schema {
  query: Query
}

type Query {
  a: A @const(data: {a: 0, d: {e: 1}, f: true})
}

type A {
  a: Int
  d: D @modify(omit: true)
  bc: BC @const(data: {d: "{{value.d}}", f: "{{value.f}}"})
}

type BC {
  d: D
  f: Boolean
}

type D {
  e: Int
}
```

#### assert:

```yml
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: query { a  { bc {d {e}, f } }}
```
