---
error: true
---

# test-response-header-value

```yaml @config
server:
  headers:
    custom: [{key: "a", value: "a \n b"}]
```

```graphql @schema
schema {
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
