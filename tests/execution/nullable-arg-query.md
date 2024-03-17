# Nullable arg query

```graphql @server
schema @server @upstream {
  query: Query
}

type Query {
  users(id: ID): [User]
    @http(baseURL: "http://jsonplaceholder.typicode.com", path: "/users", query: [{key: "id", value: "{{args.id}}"}])
}

type User {
  id: ID!
  name: String!
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users
    body: null
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
    body: null
  response:
    status: 200
    body:
      - id: 1
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { users { id } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { users(id: 1) { id } }"
```
