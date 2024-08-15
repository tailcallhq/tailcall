---
skip: true
---

# Test field aliasing.

TODO: Skipped because Tailcall does not send the alias to the remote server.

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
  dob: String!
}
```

```yml @mock
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { customer: user(id: 4) { id name date_of_birth: dob } }" }'
  expectedHits: 1
  response:
    status: 200
    body:
      data:
        user: # TODO should we alias it ????
          id: 4
          name: Tailcall
          dob: "2000-01-01"
```

```yml @test
# Positive
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

# Negative: non existent field alias
# TODO: Tailcall should return error indicating extra field (current: skip unknown fields)
# - method: POST
#   url: http://localhost:8080/graphql
#   body:
#     query: |
#       {
#         customer: user(id: 4) {
#           id
#           name
#           dob: missing_field
#         }
#       }
```