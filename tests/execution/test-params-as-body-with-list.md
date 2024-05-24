# Http with args as body with list

```graphql @config
schema @server(port: 8000) @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Query {
  firstUser(id: Int, name: String, q: [Int!]!): User @http(method: POST, path: "/users", body: "{{.args}}")
}

type User {
  id: Int
  name: String
  numbers: [Int]
}
```

```yml @mock
- request:
    method: POST
    url: http://jsonplaceholder.typicode.com/users
    body: {"id": 1, "name": "foo", "q": [1, 2, 3]}
  response:
    status: 200
    body:
      id: 1
      name: foo
      numbers: [1, 2, 3]
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |-
      {
        firstUser(id: 1, name:"foo", q: [1,2,3]) {
          id
          name
          numbers
        }
      }
```
