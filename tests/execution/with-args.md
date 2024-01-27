# With args

#### server:
```graphql
schema {
  query: Query
}

type User {
  name: String
}

type Query {
  user(id: Int!): [User]
    @http(path: "/users", query: [{key: "id", value: "{{args.id}}"}], baseURL: "http://jsonplaceholder.typicode.com")
}
```

#### assert:
```yml
mock:
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users?id=1
    headers: {}
    body: null
  response:
    status: 200
    headers: {}
    body:
    - id: 1
      name: Leanne Graham
assert:
- request:
    method: POST
    url: http://localhost:8080/graphql
    headers: {}
    body:
      query: 'query { user(id: 1) { name } }'
env: {}
```
