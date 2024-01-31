# test-enum

#### server:

```graphql
schema @server @upstream(baseURL: "http://jsonplacheholder.typicode.com") {
  query: Query
}

enum Foo {
  BAR
  BAZ
}

type Query {
  foo: Foo @http(path: "/foo")
}
```
