# test-interface-result

#### server:

```graphql
schema @server @upstream(baseURL: "http://jsonplacheholder.typicode.com") {
  query: Query
}

interface IA {
  a: String
}

type B implements IA {
  a: String
  b: String
}

type Query {
  bar: IA @http(path: "/user")
}
```
