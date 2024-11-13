---
error: true
---

# Test merge error

```graphql @schema
schema {
  query: Query
}

type Query {
  hi: Foo @expr(body: {a: "world"})
}

type Foo {
  a: String
}
```

```graphql @schema
schema {
  query: Query
}

type Query {
  hi: Foo @expr(body: {a: "world"})
}

type Foo {
  a: Int
}
```
