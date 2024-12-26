---
identity: true
---

# test-omit

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

type Query @addField(name: "foo", path: ["foo", "a", "b"]) {
  foo: Foo @http(url: "http://jsonplaceholder.typicode.com/foo") @omit
}
```
