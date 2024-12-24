# Test field aliasing

```yaml @config
server:
  port: 8001
  hostname: "0.0.0.0"
  queryValidation: false
upstream:
  httpCache: 42
```

```graphql @schema
schema @server(port: 8001, queryValidation: false, hostname: "0.0.0.0") @upstream(httpCache: 42) {
  query: Query
}

type Query {
  user(id: ID!): User! @http(url: "http://upstream/user", query: [{key: "id", value: "{{.args.id}}"}])
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
