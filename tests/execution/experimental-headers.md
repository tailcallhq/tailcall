# Experimental headers

```yaml @config
server:
  headers:
    experimental:
      - "x-tailcall"
      - "X-experimental"
```

```graphql @schema
schema {
  query: Query
}

type Query {
  users: [User] @http(url: "http://jsonplaceholder.typicode.com/users")
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
