---
identity: true
---

# test-custom-scalar

```graphql @config
schema @server @upstream {
  query: Query
}

scalar Json

type Query {
  foo: [Json] @http(url: "http://jsonplacheholder.typicode.com/foo")
}
```
