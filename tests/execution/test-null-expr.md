# Test Null expr

```graphql @server
schema {
  query: Query
}

type Query {
  hello: Int @expr(body: null)
}
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { hello }"
```
