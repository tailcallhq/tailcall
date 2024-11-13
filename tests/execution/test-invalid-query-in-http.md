---
error: true
---

# test-invalid-query-in-http

```graphql @config
schema {
  query: Query
}

type User {
  name: String
  id: Int
}

type Query {
  user: [User] @http(url: "http://jsonplaceholder.typicode.com/users", query: {key: "id", value: "{{.vars.id}}"})
}
```

```yml @file:config.yml
schema: {}
server:
  vars: [{key: "id", value: "1"}]
```
