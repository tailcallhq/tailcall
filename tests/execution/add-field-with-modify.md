# Add field with modify

####

```graphql @server
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

####

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

####

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
