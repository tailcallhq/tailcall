# Apollo Tracing

```graphql @server
schema @server(hostname: "0.0.0.0", port: 8000) {
  query: Query
}

type Query {
  hello: String! @http(baseURL: "http://api.com", path: "/")
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
