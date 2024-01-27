# Resolve with vars

#### server:
```graphql
schema @server(vars: [{key: "id", value: "1"}]) {
  query: Query
}

type User {
  name: String
  id: Int
}

type Query {
  user: [User]
    @http(path: "/users", query: [{key: "id", value: "{{vars.id}}"}], baseURL: "http://jsonplaceholder.typicode.com")
}
```

#### assert:
```yml
mock:
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
      name: Leanne Graham
assert:
- request:
    method: POST
    url: http://localhost:8080/graphql
    headers: {}
    body:
      query: 'query { user(id: 1) { name } }'
env: {}
```
