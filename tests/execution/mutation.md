# Mutation

####

```graphql @server
schema @server @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
  mutation: Mutation
}

input PostInput {
  body: String
  title: String
  userId: Int
}

type Mutation {
  insertPost(input: PostInput): Post @http(body: "{{args.input}}", method: "POST", path: "/posts")
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

####

```yml @mock
- request:
    method: POST
    url: http://jsonplaceholder.typicode.com/posts
    body: '{"body":"post-body","title":"post-title","userId":1}'
  response:
    status: 200
    body:
      body: post-body
      title: post-title
      userId: 1
```

####

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: 'mutation { insertPost(input: { body: "post-body", title: "post-title", userId: 1 }) { body } }'
```
