---
identity: true
---

# test-server-vars

```graphql @schema
schema @server(vars: [{key: "foo", value: "bar"}]) @upstream {
  query: Query
}

type Query {
  foo: String @http(url: "http://jsonplaceholder.typicode.com/foo")
}
```
