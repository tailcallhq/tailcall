# Cors allow cred true

```graphql @server
schema @server(headers: {cors: {allowCredentials: true, allowMethods: ["OPTIONS", "POST", "GET"], allowOrigins: ["abc.com", "xyz.com"], exposeHeaders: [""], maxAge: 23, vary: ["origin", "access-control-request-method", "access-control-request-headers"]}}) @upstream(baseURL: "http://example.com", batch: {delay: 1, headers: [], maxSize: 1000}) {
  query: Query
}

type Query {
  val: Int @const(data: 1)
}
```

```yml @assert
# the same request to validate caching
- method: OPTIONS
  url: http://localhost:8080/graphql
  body:
    headers:
      access-control-allow-method: "OPTIONS, POST, GET"
      access-control-allow-credentials: true
    query: "query { val }"
```
