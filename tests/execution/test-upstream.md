---
identity: true
---

# test-upstream

```graphql @schema
schema {
  query: Query
}

type Query {
  hello: String @http(url: "http://localhost:8000/hello")
}
```

```yml @config
schema: {}
upstream:
  proxy: {url: "http://localhost:8085"}
```
