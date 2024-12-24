---
skip: true
---

# Test variables.

TODO: Skipped because we do not check that variables are defined

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

type Counter {
  count: Int!
}
```

```yml @mock
- request:
    method: GET
    url: http://upstream/profiles?handles=user-1
  expectedHits: 1
  response:
    status: 200
    body:
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
