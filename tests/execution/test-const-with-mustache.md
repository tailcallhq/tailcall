# Test const with mustache

#### server:

```graphql
schema {
  query: Query
}

type Query {
  a: A @const(data: {a: 0, b: 1, c: 2})
}

type A {
  a: Int
  b: Int @modify(omit: true)
  c: Int @modify(omit: true)
  bc: BC @const(data: "{\"b\": {{value.b}}, \"c\": {{value.c}}}")
}

type BC {
  b: Int
  c: Int
}
```

#### assert:

```yml
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: query { a { bc { b c } } }
```
