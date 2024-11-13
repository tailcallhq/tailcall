# Test schema inspection with false flag

```graphql @schema
schema {
  query: Query
}

type Query {
  me: User! @http(url: "http://upstream/me")
}

type User {
  id: String
  name: String
  birthday: Date
}
```

```yml @config
schema: {}
server:
  introspection: false
upstream:
  httpCache: 42
```

```yml @mock
- request:
    method: GET
    url: http://upstream/me
  response:
    status: 200
    body:
      id: 1
      name: "John Smith"
      birthday: "2023-03-08T12:45:26-05:00"
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
        me {
          id
          name
          birthday
        }
      }
```
