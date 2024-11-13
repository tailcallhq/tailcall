---
error: true
---

# test-hostname-faliure

```graphql @config
schema @link(src: "config.yml", type: Config) {
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
server:
  hostname: "abc"
```

