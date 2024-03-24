# Graphql datasource

```graphql @server
schema @server @upstream {
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
    body: '{ "query": "query { users { name } }" }'
  response:
    status: 200
    body:
      data:
        users:
          - name: Leanne Graham
          - name: Ervin Howell
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { users_list { name } }
```
