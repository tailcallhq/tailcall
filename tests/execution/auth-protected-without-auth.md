---
expect_validation_error: true
---

# Using @protected operator without specifying server.auth config

```graphql @server
schema {
  query: Query
}

type Query {
  data: String @const(data: "data") @protected
}
```
