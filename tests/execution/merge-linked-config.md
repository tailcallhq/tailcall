# Merge linked configs

Merge should happen only on schema while configurations like schema, upstream, telemetry should be defined only by the root config

```graphql @file:link-1.graphql
schema @server(port: 3000) @upstream(httpCache: 42, batch: {delay: 22}) {
  query: Query
}

type Foo {
  foo: String
}

type Query {
  foo: Foo @http(url: "http://jsonplaceholder.typicode.com/foo")
}
```

```graphql @file:link-2.graphql
schema @server(port: 4000) @upstream(httpCache: 33, batch: {delay: 48}) {
  query: Query
}

type Post {
  id: Int!
  userId: Int!
  user: User
    @graphQL(url: "http://jsonplaceholder.typicode.com", args: [{key: "id", value: "{{.value.userId}}"}], name: "user")
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
schema
  @server(port: 8000)
  @upstream(httpCache: 10, batch: {delay: 10})
  @link(src: "link-1.graphql", type: Config)
  @link(src: "link-2.graphql", type: Config) {
  query: Query
}
```
