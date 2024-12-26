---
identity: true
---

# test-modify

```graphql @schema
schema @server @upstream {
  query: Query
}

input Foo {
  bar: String
}

type Query {
  foo(input: Foo): String @http(url: "http://jsonplaceholder.typicode.com/foo") @modify(name: "data")
}
```
