---
check_identity: true
---

# test-http-headers

```graphql @server
schema @upstream(baseURL: "http://localhost:4000") {
  query: Query
}

type Query {
  foo: String @http(headers: [{key: "foo", value: "bar"}], path: "/foo")
}
```
