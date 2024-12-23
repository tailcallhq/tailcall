# test enum as argument

```graphql @schema
schema @server {
  query: Query
}

type Query {
  user(id: Int!, test: Test): User
    @http(
      url: "http://jsonplaceholder.typicode.com/users/{{.args.id}}"
      query: [{key: "enum", value: "{{.args.test}}"}]
    )
}

enum Test {
  A
  B
}

type User {
  id: Int!
  name: String!
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1?enum=A
  response:
    status: 200
    body:
      id: 1
      name: Json Schema
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { user(id: 1, test: A) { name } }"
```
