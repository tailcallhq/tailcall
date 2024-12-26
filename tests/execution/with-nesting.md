# With nesting

```graphql @schema
schema @server {
  query: Query
}

type Query {
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/1")
}

type User {
  id: Int!
  name: String!
  username: String!
  email: String!
  phone: String
  website: String
  posts: [Post] @http(url: "http://jsonplaceholder.typicode.com/users/{{.value.id}}/posts")
}

type Post {
  id: Int
  title: String
  userId: Int
  body: String
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
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
  response:
    status: 200
    body:
      - title: title1
      - title: title2
      - title: title3
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { user { posts { title } } }
```
