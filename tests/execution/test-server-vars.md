---
identity: true
---

# test-server-vars

```yaml @config
server:
  vars:
    - key: "foo"
      value: "bar"
```

```graphql @schema
schema @server @upstream {
  query: Query
}

type Query {
  foo: String @http(url: "http://jsonplaceholder.typicode.com/foo")
}
```
