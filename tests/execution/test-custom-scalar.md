# test-custom-scalar

###### check identity

####

```graphql @server
schema @server @upstream(baseURL: "http://jsonplacheholder.typicode.com") {
  query: Query
}

scalar Json

type Query {
  foo: [Json] @http(path: "/foo")
}
```
