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

#### assert:

```yml
mock:
  - request:
      method: GET
      url: https://jsonplaceholder.typicode.com/users/1
      headers: {}
      body: null
    response:
      status: 200
      headers: {}
      body:
        id: 1
        name: Leanne Graham
assert:
  - request:
      method: POST
      url: http://localhost:8080/graphql
      headers: {}
      body:
        query: query { firstUser { user1 { name } } }
env: {}
```
