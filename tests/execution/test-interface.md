---
identity: true
---

# test-interface

```graphql @schema
schema @server @upstream {
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
