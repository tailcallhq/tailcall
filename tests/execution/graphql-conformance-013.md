# Test unions
```graphql @config
schema
  @server(port: 8001, queryValidation: false, hostname: "0.0.0.0")
  @upstream(baseURL: "http://upstream/graphql", httpCache: 42) {
  query: Query
}

type Query {
  me: User! @graphQL(name: "me")
}

type User {
  id: String
  name: String
  birthday: Date
}
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
        __type(name: "User") {
          name
          fields {
            name
            type {
              name
            }
          }
        }
      }



