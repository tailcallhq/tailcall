# Sending a batched graphql request

```yaml @config
server:
  batchRequests: true
```

```graphql @schema
schema {
  query: Query
}

type Query {
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/1")
}

type User {
  id: Int
  name: String
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
    headers:
      test: test
  expectedHits: 3
  response:
    status: 200
    body:
      id: 1
      name: foo
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    - query: query { user { id } }
    - query: query { user { name } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { user { id } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: FOO
```
