---
identity: true
---

# test-cache

```graphql @config
schema @server @upstream {
  query: Query
}

type Query {
  user: User @http(url: "http://jsonplaceholder.typicode.com/foo") @cache(maxAge: 300)
}

type User @cache(maxAge: 900) {
  id: Int
  name: String
}
```
