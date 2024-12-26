# test-input-with-arg-type

```graphql @schema
schema {
  query: Query
}

type Query {
  queryTest(filter: StringFilter): [MyType]
    @graphQL(name: "getMyType", args: [{key: "filter", value: "{{.args.filter}}"}], url: "http://localhost:8083/mesh")
}

type StringFilter {
  s: String
}
type IntFilter {
  i: Int
}

type MyType {
  id: String!
  name(sf: StringFilter): String
  num(if: IntFilter): Int
}
```

```yml @mock
- request:
    method: POST
    url: http://localhost:8083/mesh
    textBody: '{ "query": "query { getMyType(filter: {s: \\\"1\\\"}) { id name(sf: {s: \\\"f\\\"}) num(if: {i: 3}) } }" }'
  response:
    status: 200
    body:
      data:
        getMyType:
          - id: 1
            name: foo
            num: 1
```

```yml @test
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: 'query { queryTest(filter: {s: "1"}) { id  name(sf: {s: "f"}) num(if: {i :3}) } }'
```
