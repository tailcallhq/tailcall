# n + 1 Request List

```yaml @config
upstream:
  batch:
    delay: 1
    maxSize: 1000
```

```graphql @schema
schema {
  query: Query
}

type Query {
  foos: [Foo] @http(url: "http://example.com/foos")
  bars: [Bar] @http(url: "http://example.com/bars")
}

type Foo {
  id: Int!
  name: String!
  bar: Bar @http(url: "http://example.com/bars", query: [{key: "fooId", value: "{{.value.id}}"}], batchKey: ["fooId"])
}

type Bar {
  id: Int!
  fooId: Int!
  foo: [Foo] @http(url: "http://example.com/foos", query: [{key: "id", value: "{{.value.fooId}}"}], batchKey: ["id"])
}
```

```yml @mock
- request:
    method: GET
    url: http://example.com/bars
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
  response:
    status: 200
    body:
      - id: 1
        name: foo1
      - id: 2
        name: foo2
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { bars { foo { id } fooId id } }
```
