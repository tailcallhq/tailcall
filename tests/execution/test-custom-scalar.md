---
identity: true
---

# test-custom-scalar

```graphql @config
schema @server @upstream(baseURL: "http://jsonplacheholder.typicode.com") {
  query: Query
}

scalar Json

type Query {
  foo: [Json] @http(path: "/foo")
}
```
