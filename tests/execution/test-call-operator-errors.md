---
expect_validation_error: true
---

# test-call-operator

```graphql @server
schema @upstream(baseURL: "http://localhost:3000") {
  query: Query
}

type Post {
  argumentMismatchGraphQL: User @call(steps: [{query: "userWithGraphQLResolver"}])
  headersMismatchGraphQL: User @call(steps: [{query: "userWithGraphQLResolver"}])
  urlMismatchHttp: User @call(steps: [{query: "user"}])
  userId: Int!
  withoutOperator: User @call(steps: [{args: {id: "{{.value.userId}}"}}])
  withoutResolver: User @call(steps: [{query: "userWithoutResolver", args: {id: "{{.value.userId}}"}}])
}

type Query {
  posts: [Post] @http(path: "/posts")
  user(id: Int!): User @http(path: "/users/{{.args.id}}")
  userWithGraphQLHeaders(id: Int!): User @graphQL(headers: [{key: "id", value: "{{.args.id}}"}], name: "user")
  userWithGraphQLResolver(id: Int!): User @graphQL(args: [{key: "id", value: "{{.args.id}}"}], name: "user")
  userWithoutResolver(id: Int!): User
}

type User {
  id: Int!
}
```
