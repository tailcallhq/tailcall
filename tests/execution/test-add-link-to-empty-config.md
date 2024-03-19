# test-add-link-to-empty-config

```graphql @file:link-const.graphql
schema @server @upstream {
  query: Query
}

type Query {
  hello: String @const(data: "Hello from server")
}
```

```graphql @file:link-enum.graphql
schema @server @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
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

```graphql @server
schema @server @upstream @link(src: "link-const.graphql", type: Config) @link(src: "link-enum.graphql", type: Config) {
  query: Query
}
```
