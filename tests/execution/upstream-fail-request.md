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
    body: null
  response:
    status: 503
    body: {}
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { user { name } }
```
