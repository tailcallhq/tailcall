# test-query-documentation

###### check identity


```graphql @server
schema @server @upstream(baseURL: "http://jsonplacheholder.typicode.com") {
  query: Query
}

type Query {
  """
  This is test
  """
  foo: String @http(path: "/foo")
}
```
