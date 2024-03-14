# Experimental headers

```graphql @server
schema @server(headers: {experimental: ["x-tailcall", "x-experimental"]}) {
  query: Query
}

type Query {
  users: [User] @http(path: "/users", baseURL: "http://jsonplaceholder.typicode.com")
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
    headers:
      x-tailcall: "tailcall-header"
      x-experimental: "experimental-header"
      x-not-allowed: "not-allowed-header"
    body: null
  response:
    status: 200
    body:
      - id: 1
        name: Leanne Graham
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  headers:
    x-tailcall: "tailcall-header"
    x-experimental: "experimental-header"
  body:
    query: query { users { id name } }
```
