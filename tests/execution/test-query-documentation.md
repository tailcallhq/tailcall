---
identity: true
---

# test-query-documentation

```graphql @config
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
