# Ref other


```graphql @server
schema @server @upstream(baseURL: "https://jsonplaceholder.typicode.com") {
  query: Query
}

type User {
  name: String
  id: Int
}

type User1 {
  user1: User @http(path: "/users/1")
}

type Query {
  firstUser: User1
}
```


```yml @mock
- request:
    method: GET
    url: https://jsonplaceholder.typicode.com/users/1
    body: null
  response:
    status: 200
    body:
      id: 1
      name: Leanne Graham
```


```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { firstUser { user1 { name } } }
```
