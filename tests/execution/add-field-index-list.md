# Sending field index list

#### server:

```graphql
schema {
  query: Query
}

type User {
  name: String
}

type Query @addField(name: "username", path: ["users", "0", "name"]) {
  users: [User] @http(path: "/users", baseURL: "http://jsonplaceholder.typicode.com")
}
```

#### mock:

```yml
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users
    body: null
  response:
    status: 200
    body:
      - id: 1
        name: Leanne Graham
```

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { username }
```
