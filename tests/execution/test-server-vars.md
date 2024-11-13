---
identity: true
---

# test-server-vars

```graphql @config
schema @link(src: "config.yml", type: Config) {
  query: Query
}

type Query {
  foo: String @http(url: "http://jsonplaceholder.typicode.com/foo")
}
```

```yml @file:config.yml
schema: {}
server:
  vars: [{key: "foo", value: "bar"}]
```
