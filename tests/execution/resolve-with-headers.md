# Resolve with headers

```graphql @server
schema @server @upstream(allowedHeaders: ["authorization"]) {
  query: Query
}

type Post {
  body: String!
  id: ID!
  title: String!
  userId: ID!
}

type Query {
  post1: Post @http(baseURL: "http://jsonplaceholder.typicode.com", path: "/posts/{{headers.authorization}}")
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts/1
    headers:
      authorization: 1
  response:
    status: 200
    headers:
      authorization: "1"
    body:
      id: 1
      title: post title
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  headers:
    authorization: "1"
  body:
    query: query { post1 { title } }
```
