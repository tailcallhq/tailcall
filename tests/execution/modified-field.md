# Modified field

####
```graphql @server
schema {
  query: Query
}

type User {
  name: String @modify(name: "fullname")
}

type Query {
  user: User @http(path: "/users/1", baseURL: "http://jsonplaceholder.typicode.com")
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
```

####
```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { user { fullname } }
```
