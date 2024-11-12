---
identity: true
---

# test-interface

```graphql @config
schema {
  query: Query
}

interface IA {
  a: String
}

type B implements IA {
  a: String
  b: String
}

type Query {
  bar: B @http(url: "http://jsonplaceholder.typicode.com/user")
}
```
