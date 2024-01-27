# Nullable arg query

#### server:
```graphql
schema {
  query: Query
}

type Query {
  users(id: ID): [User]
    @http(path: "/users", query: [{key: "id", value: "{{args.id}}"}], baseURL: "http://jsonplaceholder.typicode.com")
}

type User {
  id: ID!
  name: String!
}
```

#### assert:
```yml
mock:
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users
    headers: {}
    body: null
  response:
    status: 200
    headers: {}
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
    headers: {}
    body: null
  response:
    status: 200
    headers: {}
    body:
    - id: 1
assert:
- request:
    method: POST
    url: http://localhost:8080/graphql
    headers: {}
    body:
      query: query { users { id } }
- request:
    method: POST
    url: http://localhost:8080/graphql
    headers: {}
    body:
      query: 'query { users(id: 1) { id } }'
env: {}
```
