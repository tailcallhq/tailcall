# Ref other nested

```graphql @schema
schema @server {
  query: Query
}

type Query {
  firstUser: User1 @http(url: "http://jsonplaceholder.typicode.com/users/1")
}

type User {
  id: Int
  name: String
}

type User1 {
  user1: User2
}

type User2 {
  user2: User @http(url: "http://jsonplaceholder.typicode.com/users/1")
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
  expectedHits: 2
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
    query: query { firstUser { user1 { user2 { name } } } }
```
