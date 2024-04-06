---
expect_validation_error: true
---

# test-response-headers-name

```graphql @server
schema @server(headers: {custom: [{key: "🤣", value: "a"}]}) {
  query: Query
}

type User {
  name: String
  age: Int
}

type Query {
  user: User @expr(body: {name: "John"})
}
```
