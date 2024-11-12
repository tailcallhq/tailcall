---
identity: true
---

# test-http-url

```graphql @config
schema {
  query: Query
}

type Query {
  bar: String @http(url: "http://abc.com/bar")
  foo: String @http(url: "http://foo.com/foo")
}
```
