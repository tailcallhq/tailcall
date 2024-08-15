---
skip: true
---

# Test named fragments.

TODO: Skipped because there is a pending case to improve the discriminator.

```graphql @config
schema
  @server(port: 8001, queryValidation: false, hostname: "0.0.0.0")
  @upstream(baseURL: "http://upstream/", httpCache: 42) {
  query: Query
}

type Query {
  profiles(handles: [ID!]!): [Profile!]! @http(path: "/profiles", query: [{key: "handles", value: "{{.args.handles}}"}])
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
    url: http://upstream/profiles?handles=user-1&handles=user-2
  expectedHits: 1
  response:
    status: 200
    body:
      - id: 1
        handle: user-1
        friends:
          counter: 2
      - id: 2
        handle: user-2
        likers:
          counter: 4
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        profiles(handles: ["user-1", "user-2"]) {
          handle
          ...userFragment
          ...pageFragment
        }
      }

      fragment userFragment on User {
        friends {
          count
        }
      }

      fragment pageFragment on Page {
        likers {
          count
        }
      }
```
