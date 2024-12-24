# Test named fragments.

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
