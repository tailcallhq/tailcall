---
skip: true
---

# List of lists. Skipped because Tailcall cannot extract a list of lists.

```graphql @config
schema
  @server(port: 8001, queryValidation: false, hostname: "0.0.0.0")
  @upstream(baseURL: "http://upstream/", httpCache: 42) {
  query: Query
}

type Query {
  userGroups: [[User!]!]! @http(path: "/users")
}

type User {
  id: ID!
  name: String!
}
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
      - - id: 4
          name: user-4
        - id: 5
          name: user-5
        - id: 6
          name: user-6
```

```yml @test
# Positve
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        userGroups {
          id
          name
        }
      }
# Negative
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        userGroups {
          {
            id
            name
          }
        }
      }
```
