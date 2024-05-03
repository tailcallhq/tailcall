---
expect_validation_error: true
---

# test-undefined-query

```graphql @server
schema @upstream(baseURL: "http://jsonplacheholder.typicode.com") {
  query: Query
}

type Inner {
  id: Int!
}

type NestedUser {
  id: Int!
  inner: Inner
  name: String
}

type Post {
  id: Int
  innerIdNested: User! @http(path: "/users", query: [{key: "id", value: "{{.value.user.nested.inner.id.test}}"}])
  innerNested: User! @http(path: "/users", query: [{key: "id", value: "{{.value.user.nested.inner.test.id}}"}])
  nested: User! @http(path: "/users", query: [{key: "id", value: "{{.value.user.nested.test}}"}])
  user: User! @http(path: "/users", query: [{key: "id", value: "{{.value.test.id}}"}])
}

type Query {
  posts: [Post] @http(path: "/posts")
}

type User {
  id: Int!
  name: String
  nested: NestedUser
}
```
