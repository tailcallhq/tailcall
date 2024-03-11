# test-const-with-add-field

###### sdl error


```graphql @server
schema @server @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Query @addField(name: "name", path: ["post", "user", "name"]) {
  post: Post @http(path: "/posts/1")
}

type Post {
  id: Int
  title: String
  body: String
  userId: Int
  user: User @const(data: {id: 1, name: "user1"})
}

type User {
  id: Int
  name: String
}
```
