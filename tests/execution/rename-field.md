# Rename field

#### server:
```graphql
schema {
  query: Query
}

type User {
  name: String
}
type Query {
  person1: User @http(path: "/users/1", baseURL: "http://jsonplaceholder.typicode.com") @modify(name: "user1")
  person2: User @modify(name: "user2") @http(path: "/users/2", baseURL: "http://jsonplaceholder.typicode.com")
}
```

#### assert:
```yml
mock:
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
    headers: {}
    body: null
  response:
    status: 200
    headers: {}
    body:
      id: 1
      name: Leanne Graham
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/2
    headers: {}
    body: null
  response:
    status: 200
    headers: {}
    body:
      id: 2
      name: Ervin Howell
assert:
- request:
    method: POST
    url: http://localhost:8080/graphql
    headers: {}
    body:
      query: query { user1 { name } }
- request:
    method: POST
    url: http://localhost:8080/graphql
    headers: {}
    body:
      query: query { user2 { name } }
env: {}
```
