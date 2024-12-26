# Nullable arg query

```graphql @schema
schema {
  query: Query
}

type Query {
  users(id: ID): [User]
    @http(url: "http://jsonplaceholder.typicode.com/users", query: [{key: "id", value: "{{.args.id}}"}])
}

type User {
  id: ID!
  name: String!
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users?id
  response:
    status: 200
    body:
      - id: 1
      - id: 2
      - id: 3
      - id: 4
      - id: 5
      - id: 6
      - id: 7
      - id: 8
      - id: 9
      - id: 10
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users?id=1
  response:
    status: 200
    body:
      - id: 1
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { users { id } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { users(id: 1) { id } }"
```
