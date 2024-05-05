# Test inline index list

```graphql @server
schema {
  query: Query
}

type User {
  name: String
}

type Query @addField(name: "username", path: ["username", "0", "name"]) {
  username: [User] @http(path: "/users", baseURL: "http://jsonplaceholder.typicode.com") @modify(omit: true)
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
  url: http://localhost:8080/graphql
  body:
    query: query { username }
```
