# Recursive Type

```graphql @server
schema @upstream(baseURL: "https://jsonplaceholder.typicode.com") {
  query: Query
}

type Query {
  user: User @http(path: "/users/1")
}

type User {
  friend: User @http(path: "/friends/1")
  id: Int
  name: String
}
```

```yml @mock
- request:
    method: GET
    url: https://jsonplaceholder.typicode.com/users/1
  response:
    status: 200
    body:
      id: 1
      name: User1
- request:
    method: GET
    url: https://jsonplaceholder.typicode.com/friends/1
  expectedHits: 2
  response:
    status: 200
    body:
      id: 2
      name: User2
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { user { name id friend { name id friend { name id } } } }
```
