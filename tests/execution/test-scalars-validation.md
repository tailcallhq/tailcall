# Test scalar validation for input and output types

```graphql @config
schema @server(port: 8000, hostname: "localhost") {
  query: Query
}

type Query {
  emailInput(x: Email!): Boolean @expr(body: "{{.args.x}}")
  emailOutput: Email! @expr(body: 125)
}
```

```yml @test
# Valid value tests
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: "{ emailInput(x: 123) }"

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: "{ emailOutput }"
```
