---
error: true
---

# Test union type resolve

```graphql @schema
schema @server @upstream {
  query: Query
}

union FooBar = Bar | Foo

type Bar {
  bar: String!
}

type Foo {
  foo: String!
}

type Nested {
  bar: FooBar
  foo: FooBar
}

type Query {
  foo: FooBar @http(url: "http://jsonplaceholder.typicode.com/foo") @discriminate(field: "")
}
```
