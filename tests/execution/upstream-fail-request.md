# Simple GraphQL Request

```graphql @config
schema {
  query: Query
}

type User {
  id: Int
  name: String
}

type Query {
  user: User @http(path: "/users/1", baseURL: "http://jsonplaceholder.typicode.com")
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
  response:
    status: 503
    body: {}
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { user { name } }
```
