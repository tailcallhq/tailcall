# Add field with modify

```graphql @server
schema {
  query: Query
}

type Query @addField(name: "user1", path: ["person1", "name"]) @addField(name: "user2", path: ["person2", "name"]) {
  person1: User @http(baseURL: "http://jsonplaceholder.typicode.com", path: "/users/1")
  person2: User @http(baseURL: "http://jsonplaceholder.typicode.com", path: "/users/2")
}

type User {
  name: String
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
    body: null
  response:
    status: 200
    body:
      id: 1
      name: Leanne Graham
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/2
    body: null
  response:
    status: 200
    body:
      id: 2
      name: Ervin Howell
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { user1 }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { user2 }
```
