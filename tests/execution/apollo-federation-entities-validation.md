---
error: true
---

# Apollo federation query validation

```yaml @config
server:
  port: 8000
  enableFederation: true
upstream:
  httpCache: 42
  batch:
    delay: 100
```

```graphql @schema
schema {
  query: Query
}

type Query {
  user(id: Int!): User @http(url: "http://jsonplaceholder.typicode.com/users/{{.args.id}}")
}

type User @call(steps: [{query: "user", args: {id: "{{.args.id}}"}}]) {
  id: Int!
  name: String!
}

type Post @http(url: "http://jsonplaceholder.typicode.com/posts", query: [{key: "id", value: "{{.args.id}}"}]) {
  id: Int!
  title: String!
}
```
