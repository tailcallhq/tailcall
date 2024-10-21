---
identity: true
---

# test-nested-link

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

```graphql @file:graphql-with-link.graphql
schema @server @link(src: "link-enum.graphql", type: Config) {
  query: Query
}

type Post {
  id: Int!
  userId: Int!
  user: User @graphQL(url: "http://jsonplaceholder.typicode.com", args: [{key: "id", value: "{{.value.userId}}"}], name: "user")
}

type Query {
  post(id: Int!): Post @http(url: "http://jsonplaceholder.typicode.com/posts/{{.args.id}}")
}

type User {
  id: Int
  name: String
}
```

```graphql @config
schema @server @upstream @link(src: "graphql-with-link.graphql", type: Config) {
  query: Query
}
```
