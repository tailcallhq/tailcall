---
identity: true
---

# test-upstream

```graphql @config
schema @link(src: "config.yml", type: Config) {
  query: Query
}

type Query {
  hello: String @http(url: "http://localhost:8000/hello")
}
```

```yml @file:config.yml
schema: {}
upstream:
  proxy: {url: "http://localhost:8085"}
```
