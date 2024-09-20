# Sending field index list

```graphql @config
schema 
@server(port: 8000, routes: {graphQL: "/tailcall-gql"}) {
  query: Query
}

type User {
  name: String
}

type Query {
  users: [User] @http(path: "/users", baseURL: "http://jsonplaceholder.typicode.com")
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users
  response:
    status: 200
    body:
      - id: 1
        name: Leanne Graham
```

```yml @test
- method: POST
  url: http://localhost:8080/tailcall-gql
  body:
    query: query { users { name } }
```
