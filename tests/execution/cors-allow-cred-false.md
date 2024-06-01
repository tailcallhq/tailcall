# Cors allow cred false

```graphql @config
schema
  @upstream(baseURL: "http://example.com", batch: {delay: 1, maxSize: 1000})
  @server(
    headers: {
      cors: {
        allowHeaders: ["Authorization"]
        allowMethods: [POST, OPTIONS]
        allowOrigins: ["abc.com", "xyz.com"]
        allowPrivateNetwork: true
        maxAge: 23
      }
    }
  ) {
  query: Query
}

type Query {
  val: Int @expr(body: 1)
}
```

```yml @test
# the same request to validate caching
- method: OPTIONS
  url: http://localhost:8080/graphql
  body:
    headers:
      access-control-allow-method: "POST, OPTIONS"
      access-control-expose-headers: "Authorization"
    query: "query { val }"
```
