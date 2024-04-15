# Cors allow cred false

```graphql @server
schema @server(headers: {cors: {allowHeaders: ["Authorization"], allowMethods: ["POST", "OPTIONS"], allowOrigins: ["abc.com", "xyz.com"], allowPrivateNetwork: true, maxAge: 23, vary: ["origin", "access-control-request-method", "access-control-request-headers"]}}) @upstream(baseURL: "http://example.com", batch: {delay: 1, headers: [], maxSize: 1000}) {
  query: Query
}

type Query {
  val: Int @expr(body: 1)
}
```

```yml @assert
# the same request to validate caching
- method: OPTIONS
  url: http://localhost:8080/graphql
  body:
    headers:
      access-control-allow-method: "POST, OPTIONS"
      access-control-expose-headers: "Authorization"
    query: "query { val }"
```
