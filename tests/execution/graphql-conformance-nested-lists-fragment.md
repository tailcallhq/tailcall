# List of lists.

```graphql @config
schema
  @server(port: 8001, queryValidation: false, hostname: "0.0.0.0")
  @upstream(baseURL: "http://upstream/", httpCache: 42) {
  query: Query
}

type Query {
  users: [[Role!]!]! @http(path: "/users")
}

type User {
  id: ID!
  name: String!
  accountRef: String! @expr(body: "ref-{{.value.id}}-{{.value.name}}")
}

type Admin {
  name: String!
  region: String!
}

union Role = User | Admin
```

```yml @mock
- request:
    method: GET
    url: http://upstream/users
  expectedHits: 1
  response:
    status: 200
    body:
      - - id: 1
          name: user-1
        - id: 2
          name: user-2
        - id: 3
          name: user-3
      - - name: admin-1
          region: eu
        - name: admin-2
          region: us

```

```yml @test
# Positve
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        users {
          ... on User {
            id
            name
            accountRef
          }
          ... on Admin {
            name
            region
          }
        }
      }

```
