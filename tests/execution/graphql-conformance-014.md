# Test double query
```graphql @config
schema
  @server(port: 8001, queryValidation: false, hostname: "0.0.0.0")
  @upstream(baseURL: "http://upstream/graphql", httpCache: 42) {
  query: Query
}

type Query {
  user(id: ID!): User! @graphQL(name: "user", args: [{key: "id", value: "{{.args.id}}"}])
}

type User {
  id: ID!
  name: String!
  city: String
}
```

```yml @mock
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { user(id: 1) { id name } }" }'
  expectedHits: 1
  response:
    status: 200
    body:
      data:
        user:
          id: 1
          name: Admin
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query getUser {
        user(id: 1) {
          id
          name
        }
      }

      query getUser {
        user(id: 1) {
          id
          name
        }
      }
- method: POST
  url: http://localhost:8080/graphql
  body:
    operationName: getAdmin
    query: |
      query getAdmin {
        user(id: 1) {
          id
          name
        }
      }

      query getUser {
        user(id: 5) {
          id
          name
        }
      }



