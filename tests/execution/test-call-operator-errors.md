# test-call-operator

###### sdl error

#### server:

```graphql
schema @server @upstream(baseURL: "http://localhost:3000") {
  query: Query
}

type Query {
  posts: [Post] @http(path: "/posts")
  userWithoutResolver(id: Int!): User
  user(id: Int!): User @http(path: "/users/{{args.id}}")
  userWithGraphQLResolver(id: Int!): User @graphQL(name: "user", args: [{key: "id", value: "{{args.id}}"}])
  userWithGraphQLHeaders(id: Int!): User @graphQL(name: "user", headers: [{key: "id", value: "{{args.id}}"}])
}

type User {
  id: Int!
}

type Post {
  userId: Int!
  withoutResolver: User @call(query: "userWithoutResolver", args: {id: "{{value.userId}}"})
  withoutOperator: User @call(args: {id: "{{value.userId}}"})
  urlMismatchHttp: User @call(query: "user", args: {})
  argumentMismatchGraphQL: User @call(query: "userWithGraphQLResolver", args: {})
  headersMismatchGraphQL: User @call(query: "userWithGraphQLResolver", args: {})
}
```
