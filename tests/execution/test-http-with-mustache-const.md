# Test const with mustache

#### server:

```graphql
schema {
  query: Query
}

type Query {
  a: A @http(baseURL: "http://localhost:3000", path: "/a")
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

#### mock:

```yml
- request:
    url: http://localhost:3000/a
  response:
    status: 200
    body:
      a: 0
      d:
        e: 1
      f: true
```

#### assert:

```yml
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: query { a  { bc {d {e}, f } }}
```
