---
identity: true
---

# test-inline-list

```graphql @schema
schema @server @upstream {
  query: Query
}

type A {
  b: B
}

type B {
  c: String
}

type Foo {
  a: A
}

type Query @addField(name: "foo", path: ["foo", "a", "0", "b"]) {
  foo: [Foo] @http(url: "http://jsonplaceholder.typicode.com/foo") @modify(omit: true)
}
```
