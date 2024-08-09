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
# Positive: basic 1
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
# Positive: basic 2
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
# Positive: basic 2 re ordered
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
# Negative: missing fields
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4)
# Negative: non existent fields
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          email_address
        }

# Negative: missing input
# Disabled because async_graphql::dynamic does not perform validation
# - method: POST
#   url: http://localhost:8080/graphql
#   body:
#     query: |
#       query {
#         user {
#           id
#           name
#           city
#         }
#       }
```
