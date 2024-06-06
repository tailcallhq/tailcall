---
skipped: true
---
# default value for input Type

```graphql @config
schema {
  query: Query
}

type Query {
  abc(input: Input!): Int @expr(body: "{{.args.input}}")
}

input Input {
  value: Int = 1
}
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query {
        abc { id }
      }
```
