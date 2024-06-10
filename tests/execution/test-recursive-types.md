---
error: true
---

# test-recursive-types

```graphql @config
schema {
  query: Query
}

type Query {
  foo: Foo
}

type Foo {
  bars: [Bar]
}

type Bar {
  foo: Foo
  relatedBars: [Bar]
}
```
