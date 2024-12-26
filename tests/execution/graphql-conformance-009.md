---
skip: true
---

# Test variables.

TODO: Skipped because Tailcall does not construct the query correctly. Moreover it does not validate the query that is invalid (contains a missing field).

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
    @graphQL(url: "http://upstream/graphql", name: "profiles", args: [{key: "handles", value: "{{.args.handles}}"}])
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
