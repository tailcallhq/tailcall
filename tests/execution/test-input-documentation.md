---
identity: true
---

# test-input-type-documentation

```graphql @schema
schema @server @upstream {
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
  testDocumentation(input: Foo!): Post
    @http(url: "http://jsonplaceholder.typicode.com/posts", body: "{{.args.input}}", method: "POST")
}

type Post {
  body: String
  id: Int!
}

"""
Some Documentation
"""
type Query {
  foo: String @http(url: "http://jsonplaceholder.typicode.com/foo")
  postFromFoo(id: Int!): Post @http(url: "http://jsonplaceholder.typicode.com/posts?id={{.args.id}}")
}
```
