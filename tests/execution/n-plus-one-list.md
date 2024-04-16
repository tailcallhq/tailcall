# n + 1 Request List

```graphql @server
schema @upstream(baseURL: "http://example.com", batch: {delay: 1, maxSize: 1000}) {
  query: Query
}

type Bar {
  foo: [Foo] @http(batchKey: ["id"], path: "/foos", query: [{key: "id", value: "{{value.fooId}}"}])
  fooId: Int!
  id: Int!
}

type Foo {
  bar: Bar @http(batchKey: ["fooId"], path: "/bars", query: [{key: "fooId", value: "{{value.id}}"}])
  id: Int!
  name: String!
}

type Query {
  bars: [Bar] @http(path: "/bars")
  foos: [Foo] @http(path: "/foos")
}
```

```yml @mock
- request:
    method: GET
    url: http://example.com/bars
    body: null
  response:
    status: 200
    body:
      - fooId: 1
        id: 1
      - fooId: 1
        id: 2
      - fooId: 2
        id: 3
      - fooId: 2
        id: 4
- request:
    method: GET
    url: http://example.com/foos?id=1&id=2
    body: null
  response:
    status: 200
    body:
      - id: 1
        name: foo1
      - id: 2
        name: foo2
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { bars { foo { id } fooId id } }
```
