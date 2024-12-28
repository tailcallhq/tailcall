---
identity: true
---

# test-query-documentation

```graphql @schema
schema @server @upstream {
  query: Query
}

type Query {
  """
  This is test
  """
  foo: String @http(url: "http://jsonplaceholder.typicode.com/foo")
}
```
