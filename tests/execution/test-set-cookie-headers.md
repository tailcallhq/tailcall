# Set Cookie Header

```graphql @server
schema
  @server(port: 8080, graphiql: true, hostname: "0.0.0.0", headers: {setCookies: true})
  @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Query {
  user(id: Int!): User @http(path: "/users/{{args.id}}")
}
type User {
  id: Int!
  name: String!
  username: String!
  email: String!
  phone: String
  website: String
}
```

```yaml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1

  response:
    status: 200
    body:
      id: 1
      name: foo
```

```yaml @assert
- method: POST
  url: http://localhost:8080/graphql
  headers:
    set-cookie: foo2=bar2
  body:
    query: "query { user(id: 1) { name } }"
```
