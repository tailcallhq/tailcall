# Nesting level 3

```graphql @server
schema @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Post {
  body: String
  id: Int
  title: String
  user: User @http(path: "/users/{{value.userId}}")
  userId: Int!
}

type Query {
  post: Post @http(path: "/posts/1")
}

type Todo {
  completed: Boolean
}

type User {
  email: String!
  id: Int!
  name: String!
  phone: String
  todos: [Todo] @http(path: "/users/{{value.id}}/todos")
  username: String!
  website: String
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts/1
    body: null
  response:
    status: 200
    body:
      userId: 1
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
    body: null
  response:
    status: 200
    body:
      id: 1
      name: Leanne Graham
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1/todos
    body: null
  response:
    status: 200
    body:
      - completed: false
      - completed: false
      - completed: false
      - completed: true
      - completed: false
      - completed: false
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { post { user { todos { completed } } } }
```
