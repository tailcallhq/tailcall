
# Basic queries with modify field check

```graphql @config
schema
  @server(port: 8001, queryValidation: false, hostname: "0.0.0.0")
  @upstream(baseURL: "http://upstream/graphql", httpCache: 42) {
  query: Query
}

type Query {
  user(id: ID!): User! @graphQL(name: "user", args: [{key: "id", value: "{{.args.id}}"}])
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
    textBody: '{ "query": "query { user(id: 4) {  city name } }" }'
  expectedHits: 1
  response:
    status: 200
    body:
      data:
        user:
          city: Globe
          newName: Tailcall
```

```yml @test
# Positive: basic 1
- method: POST
  url: http://localhost:8001/graphql
  body:
    query: |
      {
        user(id: 4) {
          newName
          email 
        }
      }
```
