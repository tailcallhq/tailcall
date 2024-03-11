# Test const with mustache


```graphql @server
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


```yml @assert
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: query { a  { bc {d {e}, f } }}
```
