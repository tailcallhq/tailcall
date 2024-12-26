---
identity: true
---

# test-upstream

```yaml @config
upstream:
  proxy:
    url: "http://localhost:8085"
```

```graphql @schema
schema @server @upstream {
  query: Query
}

type Query {
  hello: String @http(url: "http://localhost:8000/hello")
}
```
