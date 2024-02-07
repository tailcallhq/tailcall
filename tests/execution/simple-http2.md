# Simple query

#### server:

```graphql
schema
  @server(cert: "../server/config/example.crt", key: "../server/config/example-ec.key", version: "HTTP2")
  @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Query {
  firstUser: User @http(path: "/users/1")
}

type User {
  id: Int
  name: String
}
```

#### mock:

```yml
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
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
  url: https://localhost:8080/graphql
  body:
    query: query { firstUser { name } }
```
