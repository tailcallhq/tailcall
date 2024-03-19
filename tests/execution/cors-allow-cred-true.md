# CorsParams 1

```graphql @server
schema
  @upstream(baseURL: "http://example.com", batch: {delay: 1, maxSize: 1000})
  @server(
    headers: {
      corsParams: {
        allowCredentials: true
        allowMethods: ["OPTIONS", "POST", "GET"]
        allowOrigin: ["abc.com", "xyz.com"]
        allowPrivateNetwork: true
        exposeHeaders: [""]
        maxAge: 23
      }
    }
  ) {
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
      access-control-allow-origin: abc.com
      access-control-allow-method: "OPTIONS, POST, GET"
      access-control-allow-credentials: true
    query: "query { val }"
```
