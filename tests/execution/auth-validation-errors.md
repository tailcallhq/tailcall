---
error: true
---

# auth multiple

```graphql @config
schema @server @upstream @link(id: "a", src: ".htpasswd_a", type: Htpasswd) {
  query: Query
}

type Query {
  default: String @expr(body: "data") @protected(id: ["a", "b", "c"])
  foo: Foo @expr(body: {bar: "baz"})
}

type Foo @protected(id: ["x"]) {
  bar: String
}
```

```text @file:.htpasswd_a
testuser1:$apr1$e3dp9qh2$fFIfHU9bilvVZBl8TxKzL/
testuser2:$2y$10$wJ/mZDURcAOBIrswCAKFsO0Nk7BpHmWl/XuhF7lNm3gBAFH3ofsuu
```
