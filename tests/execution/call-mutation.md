# Call mutation

```graphql @schema
schema {
  query: Query
  mutation: Mutation
}

input PostInput {
  body: String
  title: String
  userId: Int
}

input PostInputWithoutUserId {
  body: String
  title: String
  userId: Int
}

type Mutation {
  attachPostToFirstUser(postId: Int!): User
    @call(steps: [{mutation: "attachPostToUser", args: {postId: "{{.args.postId}}", userId: 1}}])
  attachPostToUser(userId: Int!, postId: Int!): User
    @http(
      body: "{\"postId\":{{.args.postId}}}"
      method: "PATCH"
      url: "http://jsonplaceholder.typicode.com/users/{{.args.userId}}"
    )
  insertPost(input: PostInput): Post
    @http(body: "{{.args.input}}", method: "POST", url: "http://jsonplaceholder.typicode.com/posts")
  insertPostToFirstUser(input: PostInputWithoutUserId): Post
    @call(steps: [{mutation: "insertPostToUser", args: {input: "{{.args.input}}", userId: 1}}])
  insertMockedPost: Post
    @call(steps: [{mutation: "insertPost", args: {input: {body: "post-body", title: "post-title", userId: 1}}}])
  insertPostToUser(input: PostInputWithoutUserId!, userId: Int!): Post
    @http(
      body: "{{.args.input}}"
      method: "POST"
      url: "http://jsonplaceholder.typicode.com/users/{{.args.userId}}/posts"
    )
}

type Post {
  body: String
  id: Int
  title: String
  userId: Int
}

type Query {
  firstUser: User @http(method: "GET", url: "http://jsonplaceholder.typicode.com/users/1")
  postFromUser(userId: Int!): Post @http(url: "http://jsonplaceholder.typicode.com/posts?userId={{.args.userId}}")
}

type User {
  id: Int
  name: String
  posts: [Post] @call(steps: [{query: "postFromUser", args: {userId: "{{.value.id}}"}}])
}
```

```yml @mock
- request:
    method: POST
    url: http://jsonplaceholder.typicode.com/posts
    body: {"body": "post-body", "title": "post-title", "userId": 1}
  expectedHits: 2
  response:
    status: 200
    body:
      title: post-title
      body: post-body
      userId: 1
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
  response:
    status: 200
    body:
      id: 1
      name: foo
- request:
    method: PATCH
    url: http://jsonplaceholder.typicode.com/users/1
    body: '{"postId":1}'
  response:
    status: 200
    body:
      id: 1
      name: foo
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts?userId=1
  response:
    status: 200
    body:
      - id: 1
        title: post1-title
        body: post1-body
        userId: 1
      - id: 2
        title: post2-title
        body: post2-body
        userId: 1
      - id: 3
        title: post3-title
        body: post3-body
        userId: 1
- request:
    method: POST
    url: http://jsonplaceholder.typicode.com/users/1/posts
    body: {"body": "post-body", "title": "post-title"}
  response:
    status: 200
    body:
      title: post-title
      body: post-body
      userId: 1
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: 'mutation { insertPost(input: { body: "post-body", title: "post-title", userId: 1 }) { body } }'
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { firstUser { posts { title } } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "mutation { attachPostToFirstUser(postId: 1) { name } }"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: 'mutation { insertPostToFirstUser(input: { body: "post-body", title: "post-title" }) { body } }'
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "mutation { insertMockedPost { body } }"
```
