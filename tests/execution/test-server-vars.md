---
identity: true
---

# test-server-vars

```graphql @config
schema @server(vars: [{key: "foo", value: "bar"}]) @upstream(baseURL: "http://jsonplacheholder.typicode.com") {
  query: Query
}

type Query {
  foo: String @http(path: "/foo")
}
```
