---
error: true
---

# test-response-headers-name

```graphql @config
schema @link(src: "config.yml", type: Config) {
  query: Query
}

type User {
  name: String
  age: Int
}

type Query {
  user: User @expr(body: {name: "John"})
}
```

```yml @file:config.yml
schema: {}
server:
  headers: {custom: [{key: "🤣", value: "a"}]}
```
