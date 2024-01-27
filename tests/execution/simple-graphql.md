# Simple GraphQL Request
This is a description.

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
      url: http://jsonplaceholder.typicode.com/users/1
      headers:
        test: test
    response:
      body:
        id: 1
        name: foo
assert:
  - request:
      method: POST
      url: http://localhost:8080/graphql
      body:
        query: query { user { name } }
    response:
      body:
        data:
          user:
            name: foo
  - request:
      method: POST
      url: http://localhost:8080/graphql
      body:
        query:
          foo: bar
    response:
      body:
        data: null
        errors:
          - message: "Unexpected GraphQL Request: invalid type: map, expected a string at line 1 column 9"
```
