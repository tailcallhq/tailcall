# Simple GraphQL Request

#### server:

```graphql
schema {
  query: Query
}

type User {
  id: Int
  name: String
}

type Query {
  user: User @http(path: "/users/1", baseURL: "http://jsonplaceholder.typicode.com")
}
```

#### assert:

```yml
mock:
  - request:
      method: GET
      url: http://jsonplaceholder.typicode.com/users/1
      headers:
        test: test
      body: null
    response:
      status: 200
      headers: {}
      body:
        id: 1
        name: foo
assert:
  - request:
      method: POST
      url: http://localhost:8080/graphql
      headers: {}
      body:
        query: query { user { name } }
  - request:
      method: POST
      url: http://localhost:8080/graphql
      headers: {}
      body:
        query:
          foo: bar
env: {}
```
