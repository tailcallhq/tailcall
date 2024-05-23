# With List args

```graphql @config
schema @server(queryValidation: true) @upstream(baseURL: "http://localhost:3000") {
  query: Query
}

type Query {
  f1(q: [Int!]!): T1 @http(path: "/api", query: [{key: "q", value: "{{.args.q}}"}])
}

type T1 {
  numbers: [Int]
}
```

```yml @mock
- request:
    method: GET
    url: http://localhost:3000/api?q=1,2,3
  response:
    status: 200
    body:
      numbers: [1, 2, 3]
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { f1(q: [1,2,3]) { numbers } }"
```
