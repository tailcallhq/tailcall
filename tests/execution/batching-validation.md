---
error: true
---

# batching validation

```graphql @config
schema @upstream(httpCache: 42, batch: {delay: 1}) {
  query: Query
}

type User {
  id: Int
  name: String
}

type Query {
  user(id: Int!): User
    @http(
      url: "http://jsonplaceholder.typicode.com/users"
      method: POST
      body: "{\"uId\": \"{{.args.id}}\",\"userId\":\"{{.args.id}}\"}"
      batchKey: ["id"]
    )
  userWithId(id: Int!): User
    @http(
      url: "http://jsonplaceholder.typicode.com/users"
      method: POST
      body: "{\"uId\": \"uId\",\"userId\":\"userId\"}"
      batchKey: ["id"]
    )
  userWithIdTest(id: Int!): User
    @http(
      url: "http://jsonplaceholder.typicode.com/users"
      method: PUT
      body: "{\"uId\": \"uId\",\"userId\":\"userId\"}"
      batchKey: ["id"]
    )
}
```
