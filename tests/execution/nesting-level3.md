# Nesting level 3

#### server:
```graphql
schema @server @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Query {
  post: Post @http(path: "/posts/1")
}
type Todo {
  completed: Boolean
}

type User {
  id: Int!
  name: String!
  username: String!
  email: String!
  phone: String
  website: String
  todos: [Todo] @http(path: "/users/{{value.id}}/todos")
}

type Post {
  id: Int
  title: String
  userId: Int!
  body: String
  user: User @http(path: "/users/{{value.userId}}")
}
```

#### assert:
```yml
mock:
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts/1
    headers: {}
    body: null
  response:
    status: 200
    headers: {}
    body:
      userId: 1
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
    headers: {}
    body: null
  response:
    status: 200
    headers: {}
    body:
      id: 1
      name: Leanne Graham
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1/todos
    headers: {}
    body: null
  response:
    status: 200
    headers: {}
    body:
    - completed: false
    - completed: false
    - completed: false
    - completed: true
    - completed: false
    - completed: false
assert:
- request:
    method: POST
    url: http://localhost:8080/graphql
    headers: {}
    body:
      query: query { post { user { todos { completed } } } }
env: {}
```
