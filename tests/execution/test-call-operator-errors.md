---
error: true
---

# test-call-operator

```graphql @config
schema @server @upstream(baseURL: "http://localhost:3000") {
  query: Query
}

type Query {
  posts: [Post] @http(path: "/posts")
  userWithoutResolver(id: Int!): User
  user(id: Int!): User @http(path: "/users/{{.args.id}}")
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
