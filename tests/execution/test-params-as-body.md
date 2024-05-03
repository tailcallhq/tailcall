# Http with args as body

```graphql @server
schema @server(port: 8000) @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Query {
  firstUser(id: Int, name: String): User @http(body: "{{.args}}", method: "POST", path: "/users")
}

type User {
  id: Int
  name: String
}
```

```yml @mock
- request:
    method: POST
    url: http://jsonplaceholder.typicode.com/users
    body: {"id": 1, "name": "foo"}
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
    query: |-
      {
        firstUser(id: 1, name:"foo") {
          id
          name
        }
      }
```
