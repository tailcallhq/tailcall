---
error: true
---

# test-hostname-faliure

```graphql @schema
schema {
  query: Query
}

type User {
  id: Int
  name: String
}

type Query {
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/1")
}
```

```yml @config
schema: {}
server:
  hostname: "abc"
```
