# Nesting level 3

```graphql @schema
schema {
  query: Query
}

type Query {
  post: Post @http(url: "http://jsonplaceholder.typicode.com/posts/1")
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
  todos: [Todo] @http(url: "http://jsonplaceholder.typicode.com/users/{{.value.id}}/todos")
}

type Post {
  id: Int
  title: String
  userId: Int!
  body: String
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/{{.value.userId}}")
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts/1
  response:
    status: 200
    body:
      userId: 1
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
  response:
    status: 200
    body:
      id: 1
      name: Leanne Graham
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1/todos
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

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { post { user { todos { completed } } } }
```
