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

#### mock:

```yml
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
    headers:
      test: test
    body: null
  response:
    status: 200
    body:
      id: 1
      name: foo
```

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { user { name } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query:
      foo: bar
```
