---
identity: true
---

# test-input-type-documentation

```graphql @config
schema @server @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
  mutation: Mutation
}

"""
Test input documentation
"""
input Foo {
  """
  Test input field documentation
  """
  id: Int
}

type Mutation {
  testDocumentation(input: Foo!): Post @http(body: "{{.args.input}}", method: "POST", path: "/posts")
}

type Post {
  body: String
  id: Int!
}

"""
Some Documentation
"""
type Query {
  foo: String @http(path: "/foo")
  postFromFoo(id: Int!): Post @http(path: "/posts?id={{.args.id}}")
}
```
