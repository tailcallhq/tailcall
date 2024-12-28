---
error: true
---

# test-inline-error

```graphql @schema
schema {
  query: Query
}

type User {
  name: String
  address: Address
}
type Address {
  city: String
}

type Query @addField(name: "street", path: ["user", "address", "street"]) {
  user: User @http(url: "http://localhost:8000/user/1")
}
```
