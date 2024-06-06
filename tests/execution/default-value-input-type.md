---
skipped: true
---

# default value for input Type

```graphql @config
schema @upstream(baseURL: "http://abc.com") {
  query: Query
}

type Query {
  abc(input: Input!): Int @http(path: "/{{.args.input.id}}")
}

input Input {
  id: Int = 1
}
```

```yml @mock
- request:
    method: GET
    url: http://abc.com/1
  response:
    status: 200
    body:
      id: 1

- request:
    method: GET
    url: http://abc.com/2
  response:
    status: 200
    body:
      id: 2
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query {
        abc(input: {})
        abc(input: {id:2})
      }
```
