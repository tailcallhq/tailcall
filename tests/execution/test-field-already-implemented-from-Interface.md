---
expect_validation_error: true
---

# test-field-already-implemented-from-Interface

```graphql @server
schema {
  query: Query
}

interface IUser {
  id: ID
  name: String
}

type Query {
  user: User @http(baseURL: "http://localhost:8080", path: "/user/{{args.input.id}}")
}

type User implements IUser {
  userId: ID! @modify(name: "id")
  userName: String! @modify(name: "name")
}
```
