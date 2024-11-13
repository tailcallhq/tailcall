---
identity: true
---

# test-server-vars

```graphql @schema
schema {
  query: Query
}

type Query {
  foo: String @http(url: "http://jsonplaceholder.typicode.com/foo")
}
```

```yml @config
schema: {}
server:
  vars: [{key: "foo", value: "bar"}]
```
