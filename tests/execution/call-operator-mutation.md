# Call operator mutation

#### server:

```graphql
schema @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
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
  post: Post
    @call(mutation: "insertPost", args: {input: "{\"body\":\"user-body\",\"title\":\"user-title\",\"userId\":1}"})
}
```

#### mock:

```yml
- request:
    method: POST
    url: http://jsonplaceholder.typicode.com/posts
    body: '{"body":"post-body","title":"post-title","userId":1}'
  response:
    body:
      title: "post-title"
      body: "post-body"
      userId: 1
- request:
    method: POST
    url: http://jsonplaceholder.typicode.com/posts
    body: '{"body":"user-body","title":"user-title","userId":1}'
  response:
    body:
      title: "user-title"
      body: "body"
      userId: 1
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
  response:
    body:
      id: 1
      name: foo
```

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: 'mutation { insertPost(input: { body: "post-body", title: "post-title", userId: 1 }) { body } }'
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { firstUser { post { title } } }"
```
