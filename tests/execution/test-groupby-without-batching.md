---
error: true
---

# test-groupby-without-batching

```graphql @config
schema @link(src: "config.yml", type: Config) {
  query: Query
}

type User {
  id: Int
  name: String
}

type Query {
  user(id: Int!): User
    @http(
      url: "http://jsonplaceholder.typicode.com/users"
      query: [{key: "id", value: "{{.args.id}}"}]
      batchKey: ["id"]
    )
}
```

```yml @file:config.yml
schema: {}
upstream:
  httpCache: 42
```
