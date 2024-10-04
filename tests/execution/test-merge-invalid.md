---
error: true
---

# Test merge error

```graphql @config
schema @server @upstream(baseURL: "http://abc.com") {
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
schema @server {
  query: Query
}

type Query {
  hi: Foo @expr(body: {a: "world"})
}

type Foo {
  a: Int
}
```
