# Http with args as body

#### server:

```graphql
schema @server(port: 8000, graphiql: true) @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Query {
  firstUser(id: Int, name: String): User @http(method: POST, path: "/users", body: "{{args}}")
}

type User {
  id: Int
  name: String
}
```

#### assert:

```yml
mock:
  - request:
      method: POST
      url: http://jsonplaceholder.typicode.com/users
      headers: {}
      body: '{"id":1,"name":"foo"}'
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
        query: |-
          {
            firstUser(id: 1, name:"foo") {
              id
              name
            }
          }
env: {}
```
