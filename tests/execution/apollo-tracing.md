# Apollo Tracing

```graphql @server
schema
  @server(port: 8000, graphiql: true, hostname: "0.0.0.0")
  @telemetry(export: {apollo: {apiKey: "<api_key>", graphRef: "tailcall-demo-3@current"}}) {
  query: Query
}

type Query {
  hello: String! @http(path: "/", baseURL: "http://api.com")
}
```

```yml @mock
- request:
    method: GET
    url: http://api.com
  response:
    status: 200
    body: hello
```

```yml @test
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: query { hello }
```
