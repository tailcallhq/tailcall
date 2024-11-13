---
error: true
---

# test-experimental-headers-error

```graphql @config
schema @link(src: "config.yml", type: Config) {
  query: Query
}

type Query {
  hello: String @expr(body: "World!")
}
```

```yml @file:config.yml
schema: {}
server:
  headers: {experimental: ["non-experimental", "foo", "bar", "tailcall"]}
```
