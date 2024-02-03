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

type Query {
  users_list: [User] @graphQL(baseURL: "http://upstream/graphql", name: "users")
}
```

#### mock:

```yml
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

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { users_list { name } }
```
