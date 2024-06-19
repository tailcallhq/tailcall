# test-input-type

```graphql @config
schema {
  query: Query
}

type Query {
  queryTest(filter: Filter): [MyType]
    @graphQL(
      name: "getMyType"
      args: [{key: "filter", value: "{{.args.filter}}"}]
      baseURL: "http://localhost:8083/mesh"
    )
}

type Filter {
  a: Int
}

type MyType {
  id: String!
  name: String
}
```

```yml @mock
- request:
    method: POST
    url: http://localhost:8083/mesh
    textBody: '{ "query": "query { getMyType { id } }" }'
  response:
    status: 200
    body:
      data:
        getMyType:
          - id: 1
```

```yml @test
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: "query { queryTest { id } }"
```
