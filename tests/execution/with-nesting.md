# With nesting

#### server:

```graphql
schema @server @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Query {
  user: User @http(path: "/users/1")
}

type User {
  id: Int!
  name: String!
  username: String!
  email: String!
  phone: String
  website: String
  posts: [Post] @http(path: "/users/{{value.id}}/posts")
}

type Post {
  id: Int
  title: String
  userId: Int
  body: String
}
```

#### assert:

```yml
mock:
  - request:
      method: GET
      url: http://jsonplaceholder.typicode.com/users/1
      headers: {}
      body: null
    response:
      status: 200
      headers: {}
      body:
        email: leanne@mail.com
        id: 1
        name: Leanne Graham
        username: Bret
  - request:
      method: GET
      url: http://jsonplaceholder.typicode.com/users/1/posts
      headers: {}
      body: null
    response:
      status: 200
      headers: {}
      body:
        - title: title1
        - title: title2
        - title: title3
assert:
  - request:
      method: POST
      url: http://localhost:8080/graphql
      headers: {}
      body:
        query: query { user { posts { title } } }
env: {}
```
