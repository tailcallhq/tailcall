---
identity: true
---

# test-enum-aliases

```graphql @schema
schema {
  query: Query
}

enum Foo {
  BAR @alias(options: ["OP1", "OP2"])
  BAZ
}

type Query {
  foo: Foo @expr(body: "OP1")
}
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { foo }"
```
