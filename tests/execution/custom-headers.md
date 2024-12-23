# Custom Headers

```graphql @schema
schema @server(headers: {custom: [{key: "x-id", value: "1"}, {key: "x-name", value: "John Doe"}]}) @upstream {
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
