# Resolve with headers

```yaml @config
upstream:
  allowedHeaders:
    - authorization
```

```graphql @schema
schema {
  query: Query
}

type Post {
  id: ID!
  title: String!
  body: String!
  userId: ID!
}

type Query {
  post1: Post @http(url: "http://jsonplaceholder.typicode.com/posts/{{.headers.authorization}}")
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

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  headers:
    authorization: "1"
  body:
    query: query { post1 { title } }
```
