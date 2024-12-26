# Simple query

```graphql @schema
schema @server @upstream {
  query: Query
}

type Query {
  firstUser: User @http(url: "http://jsonplaceholder.typicode.com/users/1")
}

type User {
  id: Int
  name: String
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
      name: foo
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { firstUser { name } }
```
