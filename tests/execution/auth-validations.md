---
error: true
---

# auth multiple

```yaml @config
links:
  - id: a
    src: .htpasswd_a
    type: Htpasswd
```

```graphql @schema
schema {
  query: Query
}

type Query {
  default: String @expr(body: "data") @protected(id: ["a", "b", "c"])
  foo: Foo @expr(body: {bar: "baz"})
}

type Foo @protected(id: ["x"]) {
  bar: String
  baz: String @protected(id: ["y"])
}

type Zoo {
  a: String @protected(id: ["z"])
}

type Baz {
  x: String @protected(id: ["z"])
  y: String @protected(id: ["y"])
  z: String @protected(id: ["x"])
}
```

```text @file:.htpasswd_a
testuser1:$apr1$e3dp9qh2$fFIfHU9bilvVZBl8TxKzL/
testuser2:$2y$10$wJ/mZDURcAOBIrswCAKFsO0Nk7BpHmWl/XuhF7lNm3gBAFH3ofsuu
```
