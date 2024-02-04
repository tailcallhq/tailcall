# Caching Parent Not Null

#### server:

```graphql
schema @upstream(baseURL: "http://example.com", batch: {delay: 1, maxSize: 1000}) {
  query: Query
}

type Query {
  bars: [Bar] @http(path: "/bars")
}

type Foo {
  id: Int
}

type Bar {
  id: Int!
  foo: Foo @http(path: "/foo?id={{value.id}}") @cache(maxAge: 300)
}
```

#### mock:

```yml
- request:
    method: GET
    url: http://example.com/bars
    body: null
  response:
    status: 200
    body:
      - id: 1
      - id: 2
      - id: 3
      - id: 4
- request:
    method: GET
    url: http://example.com/foo?id=1
    body: null
  response:
    status: 200
    body:
      id: 1
- request:
    method: GET
    url: http://example.com/foo?id=2
    body: null
  response:
    status: 200
    body:
      id: 2
- request:
    method: GET
    url: http://example.com/foo?id=3
    body: null
  response:
    status: 200
    body:
      id: 3
- request:
    method: GET
    url: http://example.com/foo?id=4
    body: null
  response:
    status: 200
    body:
      id: 4
```

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { bars { foo { id } id } }
```
