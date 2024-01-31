# test-multi-interface

#### server:

```graphql
schema @server @upstream(baseURL: "http://jsonplacheholder.typicode.com") {
  query: Query
}

interface IA {
  a: String
}

interface IB {
  b: String
}

type B implements IA & IB {
  a: String
  b: String
}

type Query {
  bar: B @http(path: "/user")
}
```
