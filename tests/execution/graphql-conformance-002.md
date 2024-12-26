---
skip: true
---

# Test complex nested query.

TODO: Skipped because Tailcall does not send the whole query to the remote server. It sends a shallow version of the query.

```yaml @config
server:
  port: 8001
  hostname: "0.0.0.0"
  queryValidation: false
upstream:
  httpCache: 42
```

```graphql @schema
schema {
  query: Query
}

type Query {
  user(id: ID!): User!
    @graphQL(url: "http://upstream/graphql", name: "user", args: [{key: "id", value: "{{.args.id}}"}])
}

type User {
  id: ID!
  name: String!
  city: String
  birthday: BirthDay!
  friends: [User!]!
}

type BirthDay {
  day: Int!
  month: Int!
  year: Int
}
```

```yml @mock
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { user(id: 4) { id name city birthday { day month } friends { id name birthday { year } } } }" }'
  response:
    status: 200
    body:
      data:
        user:
          id: 4
          name: Tailcall
          city: Globe
          birthday:
            day: 15
            month: 6
          friends:
            - id: 1
              name: Person 1
              birthday:
                year: null
            - id: 2
              name: Person 2
              birthday:
                year: 2000
```

```yml @test
# Positive
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
        user(id: 4) {
          id
          name
          city
          birthday {
            day
            month
          }
          friends {
            id
            name
            birthday {
              year
            }
          }
        }
      }

# Negative: invalid selection at nested
# TODO: Tailcall should return error indicating extra field (current: skip unknown fields)
# - method: POST
#   url: http://localhost:8080/graphql
#   body:
#     query: |
#       {
#         user(id: 4) {
#           id
#           name
#           city
#           birthday {
#             day
#             month
#           }
#           friends {
#             id
#             name
#             birthday {
#               year
#               missing_field
#             }
#           }
#         }
#       }
```
