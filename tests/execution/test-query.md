---
identity: true
---

# test-query

```graphql @config
schema {
  query: Query
}

type Query {
  foo: String @http(url: "http://jsonplaceholder.typicode.com/foo")
}
```
