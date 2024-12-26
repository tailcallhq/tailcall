# Test inline fragments.

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
  profiles(handles: [ID!]!): [Profile!]!
    @http(url: "http://upstream/profiles", query: [{key: "handles", value: "{{.args.handles}}"}])
}

interface Profile {
  id: ID!
  handle: String!
}

type User implements Profile {
  id: ID!
  handle: String!
  friends: Counter!
}

type Page implements Profile {
  id: ID!
  handle: String!
  likers: Counter!
}

type Event {
  id: ID!
  handle: String!
}

type Counter {
  count: Int!
}
```

```yml @mock
- request:
    method: GET
    url: http://upstream/profiles?handles=user-1&handles=user-2
  expectedHits: 1
  response:
    status: 200
    body:
      - User:
          id: 1
          handle: user-1
          friends:
            count: 2
      - Page:
          id: 2
          handle: page-1
          likers:
            count: 4
# - request:
#     method: GET
#     url: http://upstream/profiles?handles=user-3&handles=user-4&handles=event-1
#   expectedHits: 1
#   response:
#     status: 200
#     body:
#       - id: 1
#         handle: user-3
#         friends:
#           count: 2
#       - id: 2
#         handle: user-4
#         likers:
#           count: 4
#       - id: 3
#         handle: event-1
```

```yml @test
# Positive
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        profiles(handles: ["user-1", "user-2"]) {
          handle
          ... on User {
            friends {
              count
            }
          }
          ... on Page {
            likers {
              count
            }
          }
        }
      }

# Negative: not expected return type
# Skipped because the order of fields is different between operating systems
# - method: POST
#   url: http://localhost:8080/graphql
#   body:
#     query: |
#       query {
#         profiles(handles: ["user-3", "user-4", "event-1"]) {
#           handle
#           ... on User {
#             friends {
#               count
#             }
#           }
#           ... on Page {
#             likers {
#               count
#             }
#           }
#         }
#       }

# Negative: not expected fragment type
# TODO: fix throw error because `Event` is not interface of `Profile`
# - method: POST
#   url: http://localhost:8080/graphql
#   body:
#     query: |
#       query {
#         profiles(handles: ["user-1", "user-2"]) {
#           handle
#           ... on User {
#             friends {
#               count
#             }
#           }
#           ... on Page {
#             likers {
#               count
#             }
#           }
#           ... on Event {
#             likers {
#               count
#             }
#           }
#         }
#       }
```
