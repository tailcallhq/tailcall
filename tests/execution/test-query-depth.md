# Query Complexity

```graphql @config
schema @server(queryDepth: 3) {
  query: Query
}

type User {
  id: Int!
  name: String!
  username: String!
  email: String!
  phone: String
  website: String
  blog: Blog @call(steps: [{query: "blog", args: {id: "{{.value.id}}"}}])
}

type Blog {
  id: Int!
  name: String!
}

type Post {
  id: Int!
  userId: Int!
  title: String!
  body: String!
  user: User @call(steps: [{query: "user", args: {id: "{{.value.userId}}"}}])
}

type Query {
  post: Post @http(path: "/posts/1", baseURL: "http://jsonplaceholder.typicode.com")
  user(id: Int!): User @http(path: "/users/{{.args.id}}", baseURL: "http://jsonplaceholder.typicode.com")
  blog(id: Int!): Blog @http(path: "/blogs/{{.args.id}}", baseURL: "http://jsonplaceholder.typicode.com")
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
  response:
    status: 200
    body:
      id: 1
      name: foo
      username: foo
      email: foo@typicode.com

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts/1
  response:
    status: 200
    body:
      id: 1
      userId: 1
      title: foo title

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/blogs/1
  expectedHits: 0
  response:
    status: 200
    body:
      id: 1
      title: foo blog
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { post { title, id, user { name username blog { id } }  } }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { post { title, id, user { name username }  } }
```
