# Graphql datasource

```yaml @config
upstream:
  batch:
    delay: 1
```

```graphql @schema
schema {
  query: Query
}

type Post {
  id: Int
  title: String
  userId: Int
  user: User @graphQL(url: "http://upstream/graphql", name: "user", args: [{key: "id", value: "{{.value.userId}}"}])
}

type User {
  id: Int
  name: String
}

type Query {
  posts: [Post] @http(url: "http://jsonplaceholder.typicode.com/posts")
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
