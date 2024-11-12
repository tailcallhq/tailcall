---
identity: true
---

# test-upstream

```graphql @config
schema @upstream(proxy: {url: "http://localhost:8085"}) {
  query: Query
}

type Query {
  hello: String @http(url: "http://localhost:8000/hello")
}
```
