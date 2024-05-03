---
expect_validation_error: true
---

# test-add-field-error

```graphql @server
schema {
  query: Query
}

type Address {
  city: String
}

type Query @addField(name: "street", path: ["user", "address", "street"]) {
  user: User @http(baseURL: "http://localhost:8000", path: "/user/1")
}

type User {
  address: Address
  name: String
}
```
