# Batched graphql request to batched upstream query

```yaml @config
server:
  batchRequests: true
upstream:
  batch:
    delay: 1
    maxSize: 100
```

```graphql @schema
schema {
  query: Query
}

type Query {
  user(id: Int!): User
    @http(
      url: "http://jsonplaceholder.typicode.com/users"
      query: [{key: "id", value: "{{.args.id}}"}]
      batchKey: ["id"]
    )
}

type User {
  id: Int
  name: String
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users?id=1&id=2
    headers:
      test: test
  response:
    status: 200
    body:
      - id: 1
        name: foo
      - id: 2
        name: bar
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    - query: "query { user(id: 1) { id name } }"
    - query: "query { user(id: 2) { id name } }"
```
