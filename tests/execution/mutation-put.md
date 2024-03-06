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

#### mock:

```yml
- request:
    method: PUT
    url: http://jsonplaceholder.typicode.com/posts/100
    body: '{"body":"abc","id":100,"title":"bar","userId":1}'
  response:
    status: 200
    body:
      body: abc
```

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: 'mutation { insertPost(input: { body: "abc", id: 100 , title: "bar", userId: 1}) { body } }'
```
