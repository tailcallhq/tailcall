# test-merge-server-sdl

####
```graphql @server
schema @server @upstream(baseURL: "http://jsonplacheholder.typicode.com") {
  query: Query
}

type Query {
  foo: [User] @http(path: "/users")
}

type User {
  id: Int
  name: String
}
```
