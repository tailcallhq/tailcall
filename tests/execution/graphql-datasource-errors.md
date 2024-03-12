# Graphql datasource

```graphql @server
schema {
  query: Query
}

type User {
  id: Int
  name: String
}

type Query {
  user(id: Int): User
    @graphQL(baseURL: "http://upstream/graphql", name: "user", args: [{key: "id", value: "{{args.id}}"}])
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
      data: null
      errors:
        - locations:
            - column: 9
              line: 1
          message: Failed to resolve user
          path:
            - user
- request:
    method: POST
    url: http://upstream/graphql
    body: '{ "query": "query { user(id: 2) { name id } }" }'
  response:
    status: 200
    body:
      data:
        user:
          id: 2
          name: null
      errors:
        - locations:
            - column: 35
              line: 1
          message: Failed to resolve name
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { user(id: 1) { name } }"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { user(id: 2) { name id } }"
```
