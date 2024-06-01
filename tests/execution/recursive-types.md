# Recursive Type

```graphql @config
schema @server @upstream(baseURL: "https://jsonplaceholder.typicode.com") {
  query: Query
}

type User {
  name: String
  id: Int
  friend: User @http(path: "/friends/1")
}

type Query {
  user: User @http(path: "/users/1")
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
