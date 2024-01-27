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

#### assert:

```yml
mock:
  - request:
      method: POST
      url: http://upstream/graphql
      headers: {}
      body: '{ "query": "query { users { name } }" }'
    response:
      status: 200
      headers: {}
      body:
        data:
          users:
            - name: Leanne Graham
            - name: Ervin Howell
assert:
  - request:
      method: POST
      url: http://localhost:8080/graphql
      headers: {}
      body:
        query: query { users_list { name } }
env: {}
```
