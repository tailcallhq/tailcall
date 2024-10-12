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
  accountRef: String! @http(path: "/refs/{{.value.id}}")
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
  response:
    status: 200
    body:
      - - User:
            id: 1
            name: user-1
        - User:
            id: 2
            name: user-2
        - User:
            id: 3
            name: user-3
      - - Admin:
            name: admin-1
            region: eu
        - Admin:
            name: admin-2
            region: us

# refs
- request:
    method: GET
    url: http://upstream/refs/1
  response:
    status: 200
    body: ref-1-user-1
- request:
    method: GET
    url: http://upstream/refs/2
  response:
    status: 200
    body: ref-2-user-2
- request:
    method: GET
    url: http://upstream/refs/3
  response:
    status: 200
    body: ref-3-user-3
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
