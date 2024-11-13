---
error: true
---

# test-response-header-value

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
server:
  headers: {custom: [{key: "a", value: "a \n b"}]}
```
