---
identity: true
---

# test-http-headers

```graphql @schema
schema {
  query: Query
}

type Query {
  foo: String @http(url: "http://localhost:4000/foo", headers: [{key: "foo", value: "bar"}])
}
```
