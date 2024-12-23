---
error: true
---

# test-call-operator

```graphql @schema
schema @server {
  query: Query
}

type Query {
  posts: [Post] @http(url: "http://localhost:3000/posts")
  userWithoutResolver(id: Int!): User
  user(id: Int!): User @http(url: "http://localhost:3000/users/{{.args.id}}")
  userWithGraphQLResolver(id: Int!): User @graphQL(name: "user", args: [{key: "id", value: "{{.args.id}}"}])
  userWithGraphQLHeaders(id: Int!): User @graphQL(name: "user", headers: [{key: "id", value: "{{.args.id}}"}])
}

type User {
  id: Int!
}

type Post {
  userId: Int!
  withoutResolver: User @call(steps: [{query: "userWithoutResolver", args: {id: "{{.value.userId}}"}}])
  withoutOperator: User @call(steps: [{args: {id: "{{.value.userId}}"}}])
  urlMismatchHttp: User @call(steps: [{query: "user"}])
  argumentMismatchGraphQL: User @call(steps: [{query: "userWithGraphQLResolver"}])
  headersMismatchGraphQL: User @call(steps: [{query: "userWithGraphQLResolver"}])
}
```
