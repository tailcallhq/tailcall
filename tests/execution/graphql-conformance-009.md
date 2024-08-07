---
skip: true
---

# Test variables. Skipped because Tailcall does not construct the query correctly. Moreover it does not validate the query that is invalid (contains a missing field).

```graphql @config
schema
  @server(port: 8001, queryValidation: false, hostname: "0.0.0.0")
  @upstream(baseURL: "http://upstream/graphql", httpCache: 42) {
  query: Query
}

type Query {
  profiles(handles: [ID!]!): [Profile!]!
    @graphQL(name: "profiles", args: [{key: "handles", value: "{{.args.handles}}"}])
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

type Counter {
  count: Int!
}
```

```yml @mock
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { profiles(handles: [\"user-1\"]) { id handle } }" }'
  expectedHits: 1
  response:
    status: 200
    body:
      data:
        profiles:
          - id: 1
            handle: user-1
```

```yml @test
# Positive
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query ($expandedInfo: Boolean) {
        profiles(handles: ["user-1"]) {
          id
          handle
          ... @include(if: $expandedInfo) {
            name
          }
        }
      }
    variables:
      expandedInfo: false
# Negative: missing
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query ($expandedInfo: Boolean) {
        profiles(handles: ["user-1"]) {
          id
          handle
          ... @include(if: $expandedInfo) {
            name
          }
        }
      }
```
