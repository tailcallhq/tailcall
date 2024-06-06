---
skipped: true
---

# default value for input Type

```graphql @config
schema @upstream(baseURL: "http://abc.com") {
  query: Query
}

type Query {
  abc(input: Input!): Int @http(path: "/foo/{{.args.input.id}}")
}

input Input {
  id: Int = 1
}
```

```yml @mock
- request:
    method: GET
    url: http://abc.com/foo/1
  response:
    status: 200
    body: 1

- request:
    method: GET
    url: http://abc.com/foo/2
  response:
    status: 200
    body: 2
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query {
        abc(input: {})
      }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query {
        abc(input: {id:2})
      }
```
