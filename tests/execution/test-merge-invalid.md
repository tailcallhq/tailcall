---
error: true
---

# Test merge error

```graphql @config
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

```graphql @config
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
