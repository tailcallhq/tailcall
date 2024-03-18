# test-invalid-server

---
expect_validation_error: true
---

```graphql @server
schema @server(port: "8000") {
  query: Query
}
```
