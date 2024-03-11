# test-nested-link

###### check identity


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


```graphql @file:graphql-with-link.graphql
schema @server @upstream(baseURL: "http://localhost:8000/graphql") @link(src: "link-enum.graphql", type: Config) {
  query: Query
}

type Post {
  id: Int!
  userId: Int!
  user: User @graphQL(args: [{key: "id", value: "{{value.userId}}"}], name: "user")
}

type Query {
  post(id: Int!): Post @http(baseURL: "http://jsonplaceholder.typicode.com", path: "/posts/{{args.id}}")
}

type User {
  id: Int
  name: String
}
```


```graphql @server
schema @server @upstream @link(src: "graphql-with-link.graphql", type: Config) {
  query: Query
}
```
