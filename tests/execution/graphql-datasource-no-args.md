# Graphql datasource

```graphql @server
schema {
  query: Query
}

type Query {
  users_list: [User] @graphQL(baseURL: "http://upstream/graphql", name: "users")
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
    textBody: {"query": "query { users { name } }"}
  response:
    status: 200
    body:
      data:
        users:
          - name: Leanne Graham
          - name: Ervin Howell
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { users_list { name } }
```
