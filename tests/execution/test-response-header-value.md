---
error: true
---

# test-response-header-value

```graphql @config
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

```yml @file:config.yml
schema: {}
server:
  headers: {custom: [{key: "a", value: "a \n b"}]}
```
