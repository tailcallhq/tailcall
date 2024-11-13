---
identity: true
---

# test-custom-scalar

```graphql @schema
schema {
  query: Query
}

scalar Json

type Query {
  foo: [Json] @http(url: "http://jsonplacheholder.typicode.com/foo")
}
```
