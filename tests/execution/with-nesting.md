# With nesting

```graphql @server
schema @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Post {
  body: String
  id: Int
  title: String
  userId: Int
}

type Query {
  user: User @http(path: "/users/1")
}

type User {
  email: String!
  id: Int!
  name: String!
  phone: String
  posts: [Post] @http(path: "/users/{{value.id}}/posts")
  username: String!
  website: String
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
    body: null
  response:
    status: 200
    body:
      email: leanne@mail.com
      id: 1
      name: Leanne Graham
      username: Bret
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1/posts
    body: null
  response:
    status: 200
    body:
      - title: title1
      - title: title2
      - title: title3
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { user { posts { title } } }
```
