# Call operator with graphQL datasource

```graphql @config
schema
  @server(port: 8000, hostname: "0.0.0.0")
  @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: 42) {
  query: Query
}

type Query {
  posts: [Post] @http(path: "/posts")
  user(id: Int!): User
    @graphQL(baseURL: "http://upstream/graphql", name: "user", args: [{key: "id", value: "{{.args.id}}"}])
}

type User {
  id: Int!
  name: String!
  username: String!
  email: String!
  phone: String
  website: String
}

type Post {
  id: Int!
  userId: Int!
  title: String!
  body: String!
  user: User @call(steps: [{query: "user", args: {id: "{{.value.userId}}"}}])
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
  expectedHits: 2
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
  expectedHits: 2
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
