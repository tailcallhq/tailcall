# Rest API


```graphql @file:operation-user.graphql
query ($id: Int!) @rest(method: POST, path: "/user/$id") {
  user(id: $id) {
    id
    name
  }
}
```


```graphql @server
schema
  @server
  @upstream(baseURL: "http://jsonplaceholder.typicode.com")
  @link(type: Operation, src: "operation-user.graphql") {
  query: Query
}

type Query {
  user(id: Int!): User @http(path: "/users/{{args.id}}")
}

type User {
  id: Int!
  name: String!
}
```


```yml @mock
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


```yml @assert
- method: POST
  url: http://localhost:8080/api/user/1
```
