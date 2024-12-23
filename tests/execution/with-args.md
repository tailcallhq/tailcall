# With args

```graphql @schema
schema {
  query: Query
}

type User {
  name: String
}

type Query {
  user(id: Int!): [User]
    @http(url: "http://jsonplaceholder.typicode.com/users", query: [{key: "id", value: "{{.args.id}}"}])
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
