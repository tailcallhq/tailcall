# Env value

```graphql @schema
schema @server {
  query: Query
}

type Post {
  body: String
  id: Int
  title: String
  userId: Int!
}

type Query {
  post1: Post @http(url: "http://jsonplaceholder.typicode.com/posts/{{.env.ID}}")
  post2: Post @http(url: "http://jsonplaceholder.typicode.com/posts/{{.env.POST_ID}}")
  post3: Post @http(url: "http://jsonplaceholder.typicode.com/posts/{{.env.NESTED_POST_ID}}")
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts/1
  response:
    status: 200
    body:
      body: Post 1 body
      id: 1
      title: Post 1
      userId: 1
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts/2
  response:
    status: 200
    body:
      body: Post 2 body
      id: 2
      title: Post 2
      userId: 2
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts/3
  response:
    status: 200
    body:
      body: Post 3 body
      id: 3
      title: Post 3
      userId: 3
```

```yml @env
ID: "1"
POST_ID: "2"
NESTED_POST_ID: "3"
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { post1 {id title body userId} post2 {id title body userId} post3 {id title body userId} }
```
