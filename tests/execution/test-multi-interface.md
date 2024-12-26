---
identity: true
---

# test-multi-interface

```graphql @schema
schema @server @upstream {
  query: Query
}

interface IA {
  a: String
}

interface IB {
  b: String
}

type B implements IA & IB {
  a: String
  b: String
}

type Query {
  bar: B @http(url: "http://jsonplaceholder.typicode.com/user")
}
```
