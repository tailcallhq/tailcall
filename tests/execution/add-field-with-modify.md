# Add field with modify

#### server:
```graphql
schema {
  query: Query
}

type User {
  name: String
}
type Query @addField(name: "user1", path: ["person1", "name"]) @addField(name: "user2", path: ["person2", "name"]) {
  person1: User @http(path: "/users/1", baseURL: "http://jsonplaceholder.typicode.com")
  person2: User @http(path: "/users/2", baseURL: "http://jsonplaceholder.typicode.com")
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
      query: query { user1 }
- request:
    method: POST
    url: http://localhost:8080/graphql
    headers: {}
    body:
      query: query { user2 }
env: {}
```
