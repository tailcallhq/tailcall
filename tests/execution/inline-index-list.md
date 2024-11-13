# Test inline index list

```graphql @schema
schema {
  query: Query
}

type User {
  name: String
}

type Query @addField(name: "username", path: ["username", "0", "name"]) {
  username: [User] @http(url: "http://jsonplaceholder.typicode.com/users") @modify(omit: true)
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
