---
skip: true
---

# Test inline fragments.

TODO: Skipped because Tailcall does not send the whole query with the **fragments** to the remote server.

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
    textBody: '{ "query": "query { profiles(handles: [\"user-1\", \"user-2\"]) { handle ... on User { friends { count } } ... on Page { likers { count } } } }" }'
  expectedHits: 1
  response:
    status: 200
    body:
      data:
        profiles:
          - id: 1
            handle: user-1
            __typename: User
            friends:
              count: 2
          - id: 2
            handle: user-2
            __typename: Page
            likers:
              count: 4
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { profiles(handles: [\"user-3\", \"user-4\", \"event-1\"]) { handle ... on User { friends { count } } ... on Page { likers { count } } } }" }'
  expectedHits: 1
  response:
    status: 200
    body:
      - id: 1
        handle: user-3
        __typename: User
        friends:
          counter: 2
      - id: 2
        handle: user-4
        __typename: Page
        likers:
          counter: 4
      - id: 3
        handle: event-1
        __typename: Event
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
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        profiles(handles: ["user-3", "user-4", "event-1"]) {
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
# Negative: not expected fragment type
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
          ... on Event {
            likers {
              count
            }
          }
        }
      }
```
