# With List args

```graphql @config
schema @link(src: "config.yml", type: Config) {
  query: Query
}

type Query {
  f1(q: [Int!]!): T1 @http(url: "http://localhost:3000/api", query: [{key: "q", value: "{{.args.q}}"}])
}

type T1 {
  numbers: [Int]
}
```

```yml @file:config.yml
schema: {}
server:
  queryValidation: true
```

```yml @mock
- request:
    method: GET
    url: http://localhost:3000/api?q=1&q=2&q=3
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
