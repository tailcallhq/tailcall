---
skipped: true
---

# default value for input Type

```graphql @config
schema @upstream(baseURL: "http://abc.com") {
  query: Query
}

type Query {
  bar(input: Input = {id: 1}): Int @http(path: "/bar/{{.args.input.id}}")
}

input Input {
  id: Int!
}
```

```yml @mock
- request:
    method: GET
    url: http://abc.com/bar/1
  response:
    status: 200
    body: 1

- request:
    method: GET
    url: http://abc.com/bar/2
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
        bar
      }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query {
        bar(input: {id:2})
      }
```
