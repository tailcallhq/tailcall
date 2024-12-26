---
identity: true
---

# test-add-link-to-empty-config

```yaml @config
links:
  - src: "link-expr.graphql"
    type: Config
  - src: "link-enum.graphql"
    type: Config
```

```graphql @file:link-expr.graphql
schema @server @upstream {
  query: Query
}

type Query {
  hello: String @expr(body: "Hello from server")
}
```

```graphql @file:link-enum.graphql
schema @server @upstream {
  query: Query
}

enum Foo {
  BAR
  BAZ
}

type Query {
  foo: Foo @http(url: "http://jsonplaceholder.typicode.com/foo")
}
```

```graphql @schema
schema @server @upstream {
  query: Query
}
```
