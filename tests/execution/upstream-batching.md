# Sending requests to be batched by the upstream server

```yaml @config
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
  user(id: Int): User
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
    query: "query { u1: user(id: 1) { id } u2: user(id: 2) { id } }"
```
