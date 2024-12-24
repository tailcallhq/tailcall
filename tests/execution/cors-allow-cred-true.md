# Cors allow cred true

```yaml @config
upstream:
  batch:
    delay: 1
    maxSize: 1000
server:
  headers:
    cors:
      allowCredentials: true
      allowMethods: [OPTIONS, POST, GET]
      allowOrigins: ["abc.com", "xyz.com"]
      exposeHeaders: [""]
      maxAge: 23
```

```graphql @schema
schema {
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
      access-control-allow-method: "OPTIONS, POST, GET"
      access-control-allow-credentials: true
    query: "query { val }"
```
