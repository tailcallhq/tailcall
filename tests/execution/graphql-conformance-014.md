# Test double query

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
}
```

```yml @mock
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { user(id: 1) { id name } }" }'
  expectedHits: 1
  response:
    status: 200
    body:
      data:
        user:
          id: 1
          name: Admin
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query getUser {
        user(id: 1) {
          id
          name
        }
      }

      query getUser {
        user(id: 1) {
          id
          name
        }
      }
- method: POST
  url: http://localhost:8080/graphql
  body:
    operationName: getAdmin
    query: |
      query getAdmin {
        user(id: 1) {
          id
          name
        }
      }

      query getUser {
        user(id: 5) {
          id
          name
        }
      }
```
