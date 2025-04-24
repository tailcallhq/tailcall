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
  response:
    status: 429
    body:
      {code: "UM0018", message: "change limit exceeded", cause: "exceeded the maximum allowed number of name changes"}
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { user { name } }
```
