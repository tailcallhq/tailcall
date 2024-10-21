---
error: true
---

# test-multiple-resolvable-directives-on-field

```graphql @config
schema @server  {
  query: Query
}

type User {
  name: String
  id: Int
}

type Query {
  user1: User @expr(body: {name: "John"}) @http(url: "http://jsonplaceholder.typicode.com/users/1")
  user2: User @http(url: "http://jsonplaceholder.typicode.com/users/2") @call(steps: [{query: "something"}])
}
```
