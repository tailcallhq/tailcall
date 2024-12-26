---
error: true
---

# test-field-already-implemented-from-Interface

```graphql @schema
schema {
  query: Query
}

interface IUser {
  id: ID
  name: String
}
type User implements IUser {
  userName: String! @modify(name: "name")
  userId: ID! @modify(name: "id")
}

type Query {
  user: User @http(url: "http://jsonplaceholder.typicode.com/user/{{.args.input.id}}")
}
```
