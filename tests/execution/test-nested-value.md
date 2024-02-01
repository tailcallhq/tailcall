# test-nested-value

###### check identity

#### server:

```graphql
schema @server @upstream(baseURL: "http://jsonplacheholder.typicode.com") {
  query: Query
}

type Post {
  id: Int
  user: User! @http(path: "/users", query: [{key: "id", value: "{{value.user.id}}"}])
}

type Query {
  posts: [Post] @http(path: "/posts")
}

type User {
  id: Int!
  name: String
}
```
