# test-http-baseurl

###### check identity


```graphql @server
schema @server @upstream(baseURL: "http://abc.com") {
  query: Query
}

type Query {
  bar: String @http(path: "/bar")
  foo: String @http(baseURL: "http://foo.com", path: "/foo")
}
```
