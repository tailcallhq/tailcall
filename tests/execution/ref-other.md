# Ref other

```graphql @schema
schema @server {
  query: Query
}

type User {
  name: String
  id: Int
}

type User1 {
  user1: User @http(url: "http://jsonplaceholder.typicode.com/users/1")
}

type Query {
  firstUser: User1
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
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { firstUser { user1 { name } } }
```
