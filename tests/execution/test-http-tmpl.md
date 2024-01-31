# test-http-tmpl

#### server:

```graphql
schema @server @upstream(baseURL: "http://jsonplacheholder.typicode.com") {
  query: Query
}

type Post {
  id: Int
  user: User @http(path: "/users", query: [{key: "id", value: "{{value.userId}}"}])
  userId: Int!
}

type Query {
  posts: [Post] @http(path: "/posts")
}

type User {
  id: Int
  name: String
}
```
