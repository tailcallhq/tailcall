---
identity: true
---

# test-query-documentation

```graphql @schema
schema {
  query: Query
}

type Query {
  """
  This is test
  """
  foo: String @http(url: "http://jsonplaceholder.typicode.com/foo")
}
```
