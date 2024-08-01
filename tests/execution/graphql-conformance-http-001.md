# Basic queries with field ordering check

```graphql @config
schema
  @server(port: 8001, queryValidation: false, hostname: "0.0.0.0")
  @upstream(baseURL: "http://upstream", httpCache: 42) {
  query: Query
}

type Query {
  user(id: ID!): User! @http(path: "/user", query: [{key: "id", value: "{{.args.id}}"}])
}

type User {
  id: ID!
  name: String!
  city: String
}
```

```yml @mock
- request:
    method: GET
    url: http://upstream/user?id=4
  expectedHits: 3
  response:
    status: 200
    body:
      id: 4
      name: Tailcall
      city: Globe
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
        user(id: 4) {
          city
          name
        }
      }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          city
          name
          id
        }
      }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          name
          city
        }
      }
```
