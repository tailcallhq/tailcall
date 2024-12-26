# Test schema inspection

```yaml @config
server:
  port: 8001
  hostname: "0.0.0.0"
  queryValidation: false
upstream:
  httpCache: 42
```

```graphql @schema
schema {
  query: Query
}

type Query {
  me: User! @graphQL(url: "http://upstream/graphql", name: "me")
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
```
