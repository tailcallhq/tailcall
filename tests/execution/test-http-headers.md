# test-http-headers

###### check identity

#### server:

```graphql
schema @server @upstream(baseURL: "http://localhost:4000") {
  query: Query
}

type Query {
  foo: String @http(headers: [{key: "foo", value: "bar"}], path: "/foo")
}
```
