---
identity: true
---

# omit-many

```graphql @schema
schema @server @upstream {
  query: Query
}

type Address {
  city: String
  complements: [String]
  street: String
  zipcode: String
}

type Query {
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/1")
}

type User
  @addField(name: "zipcode", path: ["address", "zipcode"])
  @addField(name: "complements", path: ["address", "complements"]) {
  address: Address @omit
  name: String
}
```
