---
error: true
---

# test-response-headers-multi

```yaml @config
server:
  headers:
    custom: [{key: "a b", value: "a \n b"}, {key: "a c", value: "a \n b"}]
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
