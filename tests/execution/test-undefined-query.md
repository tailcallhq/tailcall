# test-undefined-query

###### sdl error


```graphql @server
schema @server @upstream(baseURL: "http://jsonplacheholder.typicode.com") {
  query: Query
}

type Post {
  id: Int
  user: User! @http(path: "/users", query: [{key: "id", value: "{{value.test.id}}"}])
  nested: User! @http(path: "/users", query: [{key: "id", value: "{{value.user.nested.test}}"}])
  innerNested: User! @http(path: "/users", query: [{key: "id", value: "{{value.user.nested.inner.test.id}}"}])
  innerIdNested: User! @http(path: "/users", query: [{key: "id", value: "{{value.user.nested.inner.id.test}}"}])
}

type Query {
  posts: [Post] @http(path: "/posts")
}

type Inner {
  id: Int!
}

type NestedUser {
  id: Int!
  name: String
  inner: Inner
}

type User {
  id: Int!
  name: String
  nested: NestedUser
}
```
