# default value for input Type

```graphql @schema
schema {
  query: Query
}

type Query {
  foo(input: Input!): Int @http(url: "http://abc.com/foo/{{.args.input.id}}")
  bar(input: Input = {id: 3}): Int @http(url: "http://abc.com/foo/{{.args.input.id}}")
}

input Input {
  id: Int = 1
}
```
