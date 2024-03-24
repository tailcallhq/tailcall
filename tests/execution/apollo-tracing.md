# Apollo Tracing

```graphql @server
schema @server(graphiql: true, hostname: "0.0.0.0", port: 8000) @upstream {
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
