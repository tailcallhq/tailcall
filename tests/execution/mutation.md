# Mutation

```graphql @schema
schema @server {
  query: Query
  mutation: Mutation
}

input PostInput {
  body: String
  title: String
  userId: Int
}

type Mutation {
  insertPost(input: PostInput): Post
    @http(body: "{{.args.input}}", method: "POST", url: "http://jsonplaceholder.typicode.com/posts")
}

type Post {
  body: String
  id: Int
  title: String
  userId: Int
}

type Query {
  firstUser: User @http(method: "GET", url: "http://jsonplaceholder.typicode.com/users/1")
}

type User {
  id: Int
  name: String
}
```

```yml @mock
- request:
    method: POST
    url: http://jsonplaceholder.typicode.com/posts
    body: {"body": "post-body", "title": "post-title", "userId": 1}
  response:
    status: 200
    body:
      body: post-body
      title: post-title
      userId: 1
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: 'mutation { insertPost(input: { body: "post-body", title: "post-title", userId: 1 }) { body } }'
```
