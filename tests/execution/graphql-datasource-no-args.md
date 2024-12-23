# Graphql datasource

```graphql @schema
schema {
  query: Query
}

type User {
  id: Int
  name: String
}

type Query {
  users_list: [User] @graphQL(url: "http://upstream/graphql", name: "users")
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
