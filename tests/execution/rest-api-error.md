# Rest API error response

```graphql @file:operation-user.graphql
query ($id: Int!) @rest(method: GET, path: "/user/$id") {
  user(id: $id) {
    id
    name
  }
}
```

```yaml @config
links:
  - type: Operation
    src: operation-user.graphql
```

```graphql @schema
schema {
  query: Query
}

type Query {
  user(id: Int!): User @http(url: "http://jsonplaceholder.typicode.com/users/{{.args.id}}")
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
  response:
    status: 200
    body: {}
```

```yml @test
- method: GET
  url: http://localhost:8080/api/user/1
```
