---
error: true
---

# Validation error for union with the same type

In some cases, when the resolved data shape does not strongly correspond to GraphQL types, the discriminator may return the first possible type or no possible types at all.

```graphql @config
schema @server @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

union FooBarBaz = Bar | Baz | Foo

type Bar {
  foo: String
}

type Baz {
  baz: String
}

type Foo {
  foo: String
}

type Query {
  fooBarBaz: FooBarBaz @http(path: "/path")
}
```
