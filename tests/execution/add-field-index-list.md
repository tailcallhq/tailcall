# Sending field index list

#### server:
```graphql
schema {
  query: Query
}

type User {
  name: String
}

type Query @addField(name: "username", path: ["users", "0", "name"]) {
  users: [User] @http(path: "/users", baseURL: "http://jsonplaceholder.typicode.com")
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
      name: Leanne Graham
assert:
- request:
    method: POST
    url: http://localhost:8080/graphql
    headers: {}
    body:
      query: query { username }
env: {}
```
