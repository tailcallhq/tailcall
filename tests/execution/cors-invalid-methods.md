---
expect_validation_error: true
---

# Cors invalid allowMethods


```graphql @server
schema
  @upstream(baseURL: "http://example.com", batch: {delay: 1, maxSize: 1000})
  @server(
    headers: {
      corsParams: {
        allowCredentials: true
      }
    }
  ) {
  query: Query
}

type Query {
    val: Int @const(data: 1)
}
```
