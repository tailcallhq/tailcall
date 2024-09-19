# Query Complexity

```graphql @config
schema @server(queryComplexity: 3) {
  query: Query
}

type User {
  id: Int!
  name: String!
  username: String!
  email: String!
  phone: String
  website: String
}

type Query {
  user: User @http(path: "/users/1", baseURL: "http://jsonplaceholder.typicode.com")
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
  response:
    status: 200
    body:
      id: 1
      name: foo
      username: foo
      email: foo@typicode.com
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { user { name, username, phone, email  } }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { user { name, username } }
```
