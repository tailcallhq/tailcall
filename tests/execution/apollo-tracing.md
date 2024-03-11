# Apollo Tracing


```graphql @server
schema
  @server(port: 8000, graphiql: true, hostname: "0.0.0.0")
  @telemetry(export: {apollo: {api_key: "<api_key>", graph_ref: "tailcall-demo-3@current"}}) {
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
    body: null
  response:
    status: 200
    body: hello
```


```yml @assert
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: query { hello }
```
