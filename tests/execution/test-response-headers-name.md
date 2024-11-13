---
error: true
---

# test-response-headers-name

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

```yml @config
schema: {}
server:
  headers: {custom: [{key: "🤣", value: "a"}]}
```
