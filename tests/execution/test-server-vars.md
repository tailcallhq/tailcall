# test-server-vars

###### check identity

#### server:

```graphql
schema @server(vars: [{key: "foo",value: "bar"}]) @upstream(baseURL: "http://jsonplacheholder.typicode.com") {
  query: Query
}

type Query {
  foo: String @http(path: "/foo")
}
```
