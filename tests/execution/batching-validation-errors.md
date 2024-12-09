---
error: true
---

# batching validations

```graphql @config
schema @upstream(httpCache: 42, batch: {delay: 1}) {
  query: Query
}

type User {
  id: Int
  name: String
}

type Query {
  userByUserIds(userIds: [Int]): [User]
    @http(
      url: "http://jsonplaceholder.typicode.com/users"
      query: [{key: "id", value: "{{.args.userIds}}"}]
      batchKey: ["id"]
    )
}
```
