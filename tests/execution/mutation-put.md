# Mutation put

#### server:

```graphql
schema @server @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
  mutation: Mutation
}

input PostInput {
  id: Int
  body: String
  title: String
  userId: Int
}

type Mutation {
  insertPost(input: PostInput!): Post @http(body: "{{args.input}}", method: "PUT", path: "/posts/{{args.input.id}}")
}

type Post {
  body: String
  id: Int
  title: String
  userId: Int
}

type Query {
  firstUser: User @http(method: "GET", path: "/users/1")
}

type User {
  id: Int
  name: String
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
        id: 1
        name: Leanne Graham
  - request:
      method: PUT
      url: http://jsonplaceholder.typicode.com/posts/100
      headers: {}
      body: '{"body":"abc","id":100,"title":"bar","userId":1}'
    response:
      status: 200
      headers: {}
      body:
        body: abc
assert:
  - request:
      method: POST
      url: http://localhost:8080/graphql
      headers: {}
      body:
        query: 'mutation { insertPost(input: { body: "abc", title: "bar", userId: 1, id: 100 }) { body } }'
env: {}
```
