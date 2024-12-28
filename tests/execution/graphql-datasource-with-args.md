# Graphql datasource

```graphql @schema
schema {
  query: Query
}

type User {
  id: Int
  name: String
}

type Post {
  id: Int
  user: User
}

type Query {
  user(id: Int): User @graphQL(url: "http://upstream/graphql", name: "user", args: [{key: "id", value: "{{.args.id}}"}])
  post(id: Int): Post @graphQL(url: "http://upstream/graphql", name: "post", args: [{key: "id", value: "{{.args.id}}"}])
}
```

```yml @mock
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
    textBody: '{ "query": "query { post(id: 1) { id user { name } } }" }'
  response:
    status: 200
    body:
      data:
        post:
          id: 1
          user:
            name: Leanne Graham
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { user(id: 1) { name } }"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { post(id: 1) { id user { name } } }"
```
