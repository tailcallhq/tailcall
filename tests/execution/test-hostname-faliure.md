---
error: true
---

# test-hostname-faliure

```graphql @config
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

```yml @file:config.yml
schema: {}
server:
  hostname: "abc"
```
