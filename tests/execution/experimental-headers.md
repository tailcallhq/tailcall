# Experimental headers

```graphql @server
schema @server(headers: {experimental: ["X-experimental", "x-tailcall"]}) {
  query: Query
}

type Query {
  users: [User] @http(baseURL: "http://jsonplaceholder.typicode.com", path: "/users")
}

type User {
  id: Int
  name: String
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users
  response:
    status: 200
    headers:
      X-tailcall: "tailcall-header"
      x-experimental: "experimental-header"
      x-not-allowed: "not-allowed-header"
    body:
      - id: 1
        name: Leanne Graham
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { users { id name } }
```
