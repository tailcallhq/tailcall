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

#### assert:

```yml
mock:
  - request:
      method: POST
      url: http://upstream/graphql
      headers: {}
      body: '{ "query": "query { user(id: 1) { name } }" }'
    response:
      status: 200
      headers: {}
      body:
        data:
          user:
            name: Leanne Graham
  - request:
      method: POST
      url: http://upstream/graphql
      headers: {}
      body: '{ "query": "query { post(id: 1) { id user { name } } }" }'
    response:
      status: 200
      headers: {}
      body:
        data:
          post:
            id: 1
            user:
              name: Leanne Graham
assert:
  - request:
      method: POST
      url: http://localhost:8080/graphql
      headers: {}
      body:
        query: "query { user(id: 1) { name } }"
  - request:
      method: POST
      url: http://localhost:8080/graphql
      headers: {}
      body:
        query: "query { post(id: 1) { id user { name } } }"
env: {}
```
