# Basic queries with field ordering check

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
    textBody: '{ "query": "query { user(id: 4) { city name } }" }'
  expectedHits: 1
  response:
    status: 200
    body:
      data:
        user:
          city: Globe
          name: Tailcall
```

```yml @test
# Positive: basic 1
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/graphql
  body: |
    query {
      user(id: 4) {
        city
        name
      }
    }
```
