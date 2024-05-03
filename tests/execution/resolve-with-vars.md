# Resolve with vars

```graphql @server
schema @server(vars: [{key: "id", value: "1"}]) {
  query: Query
}

type Query {
  user: [User] @http(baseURL: "http://jsonplaceholder.typicode.com", path: "/users", query: [{key: "id", value: "{{.vars.id}}"}])
}

type User {
  id: Int
  name: String
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users?id=1
  response:
    status: 200
    body:
      - id: 1
        name: Leanne Graham
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { user(id: 1) { name } }"
```
