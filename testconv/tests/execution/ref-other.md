# Ref other

#### server:

```graphql
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

#### mock:

```yml
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

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { firstUser { user1 { name } } }
```
