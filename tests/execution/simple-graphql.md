# Simple GraphQL Request

```graphql @server
schema @server @upstream {
  query: Query
}

type Query {
  user: User @http(baseURL: "http://jsonplaceholder.typicode.com", path: "/users/1")
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
    body: null
  response:
    status: 200
    body:
      id: 1
      name: foo
```

```yml @assert
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
