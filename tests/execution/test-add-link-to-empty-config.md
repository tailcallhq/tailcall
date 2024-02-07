# test-add-link-to-empty-config

###### check identity

#### file:link-const.graphql
```graphql
schema @server @upstream {
  query: Query
}

type Query {
  hello: String @const(data: "Hello from server")
}
```

#### file:link-enum.graphql
```graphql
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

#### server:

```graphql
schema @server @upstream @link(src: "link-const.graphql", type: Config) @link(src: "link-enum.graphql", type: Config) {
  query: Query
}
```
