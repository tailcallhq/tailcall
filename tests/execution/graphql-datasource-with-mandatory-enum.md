# Graphql datasource

```graphql @config
schema {
  query: Query
}

enum EnumType {
  INFORMATION
  WARNING
}

type WithMandatoryEnum {
  type: EnumType!
}

type Post {
  severity: WithMandatoryEnum!
}

type Query {
  post: Post @expr(body: {severity: {type: null}})
}
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { post { severity { type } } }"
```
