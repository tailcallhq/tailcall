# Apollo Tracing

```graphql @schema
schema
  @server(port: 8000, hostname: "0.0.0.0")
  @telemetry(export: {apollo: {apiKey: "<api_key>", graphRef: "tailcall-demo-3@current"}}) {
  query: Query
}

type Query {
  hello: String! @http(url: "http://api.com/")
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
