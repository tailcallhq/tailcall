---
expect_validation_error: true
---

# test-response-headers-multi

```graphql @server
schema @server(headers: {custom: [{key: "a b", value: "a \n b"}, {key: "a c", value: "a \n b"}]}) {
  query: Query
}

type User {
  name: String
  age: Int
}

type Query {
  user: User @const(data: {name: "John"})
}
```
