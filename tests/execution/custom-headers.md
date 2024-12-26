# Custom Headers

```yaml @config
server:
  headers:
    custom:
      - key: "x-id"
        value: "1"
      - key: "x-name"
        value: "John Doe"
```

```graphql @schema
schema {
  query: Query
}

type Query {
  greet: String @expr(body: "Hello World!")
}
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { greet }
```
