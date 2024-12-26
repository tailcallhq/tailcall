# Simple GraphQL Request

```graphql @schema
schema {
  query: Query
}

type User {
  id: Int
  name: String
}

type Query {
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/1")
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
    headers:
      test: test
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
    query: query { user { name } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query:
      foo: bar
```
