# Empty Object Response

```graphql @server
schema {
  query: Query
}

type Company {
  id: ID
  name: String
}

type Query {
  hi(id: ID!): Company @http(baseURL: "http://localhost:3000", path: "/hi")
}
```

```yml @mock
- request:
    method: GET
    url: http://localhost:3000/hi
  response:
    status: 200
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { hi (id: 1) { name id } }"
```
