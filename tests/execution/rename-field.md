# Rename field

```graphql @schema
schema {
  query: Query
}

type User {
  name: String
}
type Query {
  person1: User @http(url: "http://jsonplaceholder.typicode.com/users/1") @modify(name: "user1")
  person2: User @modify(name: "user2") @http(url: "http://jsonplaceholder.typicode.com/users/2")
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
  response:
    status: 200
    body:
      id: 1
      name: Leanne Graham
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/2
  response:
    status: 200
    body:
      id: 2
      name: Ervin Howell
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { user1 { name } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { user2 { name } }
```
