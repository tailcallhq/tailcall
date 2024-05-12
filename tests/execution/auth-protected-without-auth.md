---
error: true
---

# Using @protected operator without specifying server.auth config

```graphql @config
schema {
  query: Query
}

type Query {
  data: String @expr(body: "data") @protected
}
```
