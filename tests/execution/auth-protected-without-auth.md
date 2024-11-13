---
error: true
---

# Using @protected operator without specifying server.auth config

```graphql @schema
schema {
  query: Query
}

type Query {
  data: String @expr(body: "data") @protected
}
```
