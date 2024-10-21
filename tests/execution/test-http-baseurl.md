---
identity: true
---

# test-http-baseurl

```graphql @config
schema @server @upstream{
  query: Query
}

type Query {
  bar: String @http(url: "http://abc.com/bar")
  foo: String @http(url: "http://foo.com/foo")
}
```
