# default value for input Type

```graphql @config
schema @upstream(baseURL: "http://abc.com") {
  query: Query
}

type Query {
  foo(input: Input!): Int @http(path: "/foo/{{.args.input.id}}")
  bar(input: Input = {id: 3}): Int @http(path: "/foo/{{.args.input.id}}")
}

input Input {
  id: Int = 1
}
```
