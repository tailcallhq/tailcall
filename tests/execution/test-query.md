---
identity: true
---

# test-query

```graphql @schema
schema {
  query: Query
}

type Query {
  foo: String @http(url: "http://jsonplaceholder.typicode.com/foo")
}
```
