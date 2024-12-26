---
identity: true
---

# test-description-many

```graphql @schema
schema @server @upstream {
  query: Query
}

type Bar {
  """
  This is test2
  """
  baz: String
}

type Query {
  """
  This is test
  """
  foo: Bar @http(url: "http://jsonplacheholder.typicode.com/foo")
}
```
