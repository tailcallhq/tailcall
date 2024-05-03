# Graphql datasource

```graphql @server
schema @upstream(batch: {delay: 1}) {
  query: Query
}

type Post {
  id: Int
  title: String
  user: User @graphQL(args: [{key: "id", value: "{{.value.userId}}"}], baseURL: "http://upstream/graphql", name: "user")
  userId: Int
}

type Query {
  posts: [Post] @http(baseURL: "http://jsonplaceholder.typicode.com", path: "/posts")
}

type User {
  id: Int
  name: String
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts
  response:
    status: 200
    body:
      - id: 1
        title: a
        userId: 1
      - id: 2
        title: b
        userId: 1
      - id: 3
        title: c
        userId: 2
      - id: 4
        title: d
        userId: 2
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { user(id: 1) { name } }" }'
  response:
    status: 200
    body:
      data:
        user:
          name: Leanne Graham
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { user(id: 2) { name } }" }'
  response:
    status: 200
    body:
      data:
        user:
          name: Ervin Howell
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { posts { title user { name } } }
```
