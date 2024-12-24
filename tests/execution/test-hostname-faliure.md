---
error: true
---

# test-hostname-faliure

```yaml @config
server:
  hostname: abc
```

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
