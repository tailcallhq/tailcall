# Use extra directives in the config

Use of directives that is not part of Tailcall directives.

```graphql @config
schema @server(port: 8000) @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Query {
  user(id: Int!): User @http(path: "/users/{{.args.id}}")
  post(id: Int!): User @http(path: "/posts/{{.args.id}}")
}

type User @call(steps: [{query: "user", args: {id: "{{.value.id}}"}}]) @shareable {
  id: Int!
  name: String!
}

type Post @expr(body: {id: "{{.value.id}}", title: "post-title-{{.value.id}}"}) {
  id: Int!
  title: String! @override(from: "name")
  body: String! @extension(a: 1, b: 2, c: 3)
}
```
