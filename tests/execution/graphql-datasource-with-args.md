# Graphql datasource

```graphql @server
schema {
  query: Query
}

type Post {
  id: Int
  user: User
}

type Query {
  post(id: Int): Post
    @graphQL(args: [{key: "id", value: "{{args.id}}"}], baseURL: "http://upstream/graphql", name: "post")
  user(id: Int): User
    @graphQL(args: [{key: "id", value: "{{args.id}}"}], baseURL: "http://upstream/graphql", name: "user")
}

type User {
  id: Int
  name: String
}
```

```yml @mock
- request:
    method: POST
    url: http://upstream/graphql
    body: '{ "query": "query { user(id: 1) { name } }" }'
  response:
    status: 200
    body:
      data:
        user:
          name: Leanne Graham
- request:
    method: POST
    url: http://upstream/graphql
    body: '{ "query": "query { post(id: 1) { id user { name } } }" }'
  response:
    status: 200
    body:
      data:
        post:
          id: 1
          user:
            name: Leanne Graham
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { user(id: 1) { name } }"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { post(id: 1) { id user { name } } }"
```
