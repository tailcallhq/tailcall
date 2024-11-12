---
identity: true
---

# test-http-headers

```graphql @config
schema {
  query: Query
}

type Query {
  foo: String @http(url: "http://localhost:4000/foo", headers: [{key: "foo", value: "bar"}])
}
```
