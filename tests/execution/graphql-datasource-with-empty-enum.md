# Graphql datasource

```graphql @config
schema {
  query: Query
}

enum EnumType {
  INFORMATION
  WARNING
}

type WithOptEnum {
  type: EnumType
}

type Post {
  severity: WithOptEnum!
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
