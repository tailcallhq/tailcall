# Test field aliasing

```graphql @config
schema
  @server(port: 8001, queryValidation: false, hostname: "0.0.0.0")
  @upstream(baseURL: "http://upstream/", httpCache: 42) {
  query: Query
}

type Query {
  user(id: ID!): User! @http(path: "/user", query: [{key: "id", value: "{{.args.id}}"}])
}

type User {
  id: ID!
  name: String!
  dob: String!
}
```

```yml @mock
- request:
    method: GET
    url: http://upstream/user?id=4
  expectedHits: 1
  response:
    status: 200
    body:
      id: 4
      name: Tailcall
      dob: "2000-01-01"
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
        customer: user(id: 4) {
          id
          name
          date_of_birth: dob
        }
      }
```
