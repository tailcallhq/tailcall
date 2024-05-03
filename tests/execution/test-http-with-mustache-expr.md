# Test expr with mustache

```graphql @server
schema {
  query: Query
}

type A {
  a: Int
  bc: BC @expr(body: {d: "{{.value.d}}", f: "{{.value.f}}"})
  d: D @modify(omit: true)
}

type BC {
  d: D
  f: Boolean
}

type D {
  e: Int
}

type Query {
  a: A @http(baseURL: "http://localhost:3000", path: "/a")
}
```

```yml @mock
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

```yml @test
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: query { a  { bc {d {e}, f } }}
```
