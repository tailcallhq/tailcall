---
error: true
---

# Apollo federation query validation

```graphql @config
schema @link(src: "config.yml", type: Config) {
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

```yml @file:config.yml
schema: {}
server:
  port: 8000
  enableFederation: true
upstream:
  httpCache: 42
  batch:
    delay: 100
```
