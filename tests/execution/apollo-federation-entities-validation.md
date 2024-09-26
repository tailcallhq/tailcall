---
error: true
---

# Apollo federation query validation

```graphql @config
schema
  @server(port: 8000)
  @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: 42, batch: {delay: 100}) {
  query: Query
}

type Query {
  user(id: Int!): User @http(path: "/users/{{.args.id}}")
}

type User @call(steps: [{query: "user", args: {id: "{{.args.id}}"}}]) {
  id: Int!
  name: String!
}

type Post @http(path: "/posts", query: [{key: "id", value: "{{.args.id}}"}]) {
  id: Int!
  title: String!
}
```
