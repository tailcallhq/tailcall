---
error: true
---

# Recursive Type no resolver check

Should throw error about missing resolver without panicking with stack overflow error.

```graphql @config
schema @server @upstream(baseURL: "https://jsonplaceholder.typicode.com") {
  query: Query
}

type User {
  name: String
  id: Int!
  connections: [Connection]
}

type Connection {
  type: String
  user: User
}

type Query {
  user: User
}
```
