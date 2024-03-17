# Call operator with graphQL datasource

```graphql @server
schema @server(graphiql: true, hostname: "0.0.0.0", port: 8000) @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: true) {
  query: Query
}

type Post {
  body: String!
  id: Int!
  title: String!
  user: User @call(query: "user", args: {id: "{{value.userId}}"})
  userId: Int!
}

type Query {
  posts: [Post] @http(path: "/posts")
  user(id: Int!): User @graphQL(args: [{key: "id", value: "{{args.id}}"}], baseURL: "http://upstream/graphql", name: "user")
}

type User {
  email: String!
  id: Int!
  name: String!
  phone: String
  username: String!
  website: String
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts
    body: null
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
    body: '{ "query": "query { user(id: 1) { name } }" }'
  expected_hits: 1
  response:
    status: 200
    body:
      data:
        user:
          name: Leanne Graham
- request:
    method: POST
    url: http://upstream/graphql
    body: '{ "query": "query { user(id: 2) { name } }" }'
  expected_hits: 1
  response:
    status: 200
    body:
      data:
        user:
          name: Ervin Howell
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { posts { title user { name } } }
```
