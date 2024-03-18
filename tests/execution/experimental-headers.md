# Experimental headers

```graphql @server
schema @server(headers: {experimental: ["X-experimental", "x-tailcall"]}) @upstream {
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
    headers:
      X-tailcall: "tailcall-header"
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
    X-tailcall: "tailcall-header"
    x-experimental: "experimental-header"
  body:
    query: query { users { id name } }
```
