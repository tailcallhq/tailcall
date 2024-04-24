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
  users_list: [User] @graphQL(baseURL: "http://upstream/graphql", name: "users")
}
```

```yml @mock
- request:
    method: POST
    url: http://upstream/graphql
    body: '{ "query": "query { users { name } }" }'
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
  headers:
    Accept: application/graphql-response+json
  body:
    query: query { users_list { name } }
```
