# Set Cookie Header

```graphql @server
schema
  @server(headers: {setCookies: true}, graphiql: true, hostname: "0.0.0.0", port: 8080)
  @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Query {
  user(id: Int!): User @http(path: "/users/{{args.id}}")
}

type User {
  email: String!
  id: Int!
  name: String!
  phone: String
  username: String!
  website: String
}
```

```yaml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1

  response:
    status: 200
    headers:
      set-cookie: user=1
    body:
      id: 1
      name: foo

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/2

  response:
    status: 200
    headers:
      set-cookie: user=2
    body:
      id: 2
      name: bar
```

```yaml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { u1:user(id: 1) { name } u2:user(id: 2) { name } }"
```
