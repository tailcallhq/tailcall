# Sending field index list

```graphql @config
schema @link(src: "config.yml", type: Config) {
  query: Query
}

type User {
  name: String
}

type Query {
  users: [User] @http(url: "http://jsonplaceholder.typicode.com/users")
}
```

```yml @file:config.yml
server:
  routes: {graphQL: "/tailcall-gql", status: "/health"}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users
  response:
    status: 200
    body:
      - id: 1
        name: Leanne Graham
```

```yml @test
- method: POST
  url: http://localhost:8080/tailcall-gql
  body:
    query: query { users { name } }

- method: GET
  url: http://localhost:8080/health
  body:
    query: query { users { name } }
```
