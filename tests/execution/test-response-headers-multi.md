---
error: true
---

# test-response-headers-multi

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
  headers: {custom: [{key: "a b", value: "a \n b"}, {key: "a c", value: "a \n b"}]}
```
