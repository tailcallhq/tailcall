# Graphql datasource

#### server:

```graphql
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
  user(id: Int): User
    @graphQL(baseURL: "http://upstream/graphql", name: "user", args: [{key: "id", value: "{{args.id}}"}])
  post(id: Int): Post
    @graphQL(baseURL: "http://upstream/graphql", name: "post", args: [{key: "id", value: "{{args.id}}"}])
}
```

#### mock:

```yml
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

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { user(id: 1) { name } }"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { post(id: 1) { id user { name } } }"
```
