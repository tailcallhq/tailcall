---
check_identity: true
---

# test-enum-description

```graphql @server
schema @server @upstream(baseURL: "http://localhost:8080") {
  query: Query
}

"""
Description of enum Foo
"""
enum Foo {
  BAR
  BAZ
}

type Query {
  foo(val: String!): Foo @expr(body: "{{.args.val}}")
}
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: 'query { foo(val: "BAR") }'

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: 'query { foo(val: "BAZ") }'

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: 'query { foo(val: "INVALID") }'
```
