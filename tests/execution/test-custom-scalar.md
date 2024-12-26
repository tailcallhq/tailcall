---
identity: true
---

# test-custom-scalar

```graphql @schema
schema @server @upstream {
  query: Query
}

scalar Json

type Query {
  foo: [Json] @http(url: "http://jsonplacheholder.typicode.com/foo")
}
```
