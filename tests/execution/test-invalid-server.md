---
expect_validation_error: true
---

# test-invalid-server

```graphql @server
schema @server(port: "8000") {
  query: Query
}
```
