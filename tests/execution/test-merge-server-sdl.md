# test-merge-server-sdl

```graphql @schema
schema @server {
  query: Query
}

type Query {
  foo: [User] @http(url: "http://jsonplaceholder.typicode.com/users")
}

type User {
  id: Int
  name: String
}
```
