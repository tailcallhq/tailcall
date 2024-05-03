# With args

```graphql @server
schema {
  query: Query
}

type Query {
  user(id: Int!): [User]
    @http(baseURL: "http://jsonplaceholder.typicode.com", path: "/users", query: [{key: "id", value: "{{.args.id}}"}])
}

type User {
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
