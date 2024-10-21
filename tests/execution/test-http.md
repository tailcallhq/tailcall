---
identity: true
---

# test-http

```graphql @config
schema @server @upstream {
  query: Query
}

type Query {
  foo: [User] @http(url: "http://jsonplaceholder.typicode.com/users")
}

type User {
  id: Int
  name: String
}
```
