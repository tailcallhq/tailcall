---
error: true
---

# test-response-headers-name

```graphql @config
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
