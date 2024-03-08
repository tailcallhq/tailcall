# Rest API

##### only

#### file:operation-user.graphql

```graphql
query ($id: Int!) @rest(method: "get", path: "/user/$id") {
  user(id: $id) {
    id
    name
  }
}

```

#### server:

```graphql
schema
  @server
  @upstream(baseURL: "http://jsonplaceholder.typicode.com")
  @link(type: RestOperation, src: "operation-user.graphql") {
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
- method: GET
  url: http://localhost:8080/api/user/1
```
