# test-directives-undef-null-fields

###### sdl error

#### server:

```graphql
schema @server(vars: [{key: "a", value: "1"}, {key: "c", value: "d"}]) {
  query: Query
}

type NestedUser {
  id: ID
}

type User {
  id: ID!
  nestedUser: NestedUser
  nonNullableNestedUser: NestedUser!
}

type Query {
  userAccessHeadersVars(id: ID!): User
    @http(path: "/user/{{args.id}}/{{headers.garbage}}/{{vars.garbage}}", baseURL: "http://localhost:8080")
  userListArg(id: [ID]): User @http(path: "/user/{{args.id}}", baseURL: "http://localhost:8080")
  userNullableArg(id: ID): User @http(path: "/user/{{args.id}}", baseURL: "http://localhost:8080")
  userUndefinedArg(id: ID): User @http(path: "/user/{{args.uid}}", baseURL: "http://localhost:8080")
}

type Post {
  id: Int!
  userId: Int
  user: User @http(path: "/users/{{value.id}}", baseURL: "http://localhost:8080")
  nonNullableUser: User! @http(path: "/users/{{value.id}}", baseURL: "http://localhost:8080")
  userArg: User @http(path: "/users/{{args.id}}", baseURL: "http://localhost:8080")
  userInvalidDirective: User @http(path: "/users/{{Vale.userId}}", baseURL: "http://localhost:8080")
  userNonScalar: User @http(path: "/users/{{value.nonNullableUser}}", baseURL: "http://localhost:8080")
  userNullable: User @http(path: "/users/{{value.user.id}}", baseURL: "http://localhost:8080")
  nestedUserNullable: User
    @http(path: "/users/{{value.nonNullableUser.nestedUser.id}}", baseURL: "http://localhost:8080")
  nestedNonScalar: User
    @http(path: "/users/{{value.nonNullableUser.nonNullableNestedUser}}", baseURL: "http://localhost:8080")
  nestedUndefinedValue: User
    @http(path: "/users/{{value.nonNullableUser.nonNullableNestedUser.userId}}", baseURL: "http://localhost:8080")
  nestedNullable: User
    @http(path: "/users/{{value.nonNullableUser.nonNullableNestedUser.id}}", baseURL: "http://localhost:8080")
  userNullValue: User @http(path: "/users/{{value.userId}}", baseURL: "http://localhost:8080")
  # nullable values are allowed in queries
  userNullValueQuery: User
    @http(path: "/users", query: [{key: "id", value: "{{value.id}}"}], baseURL: "http://localhost:8080")
  userUndefinedValue: User @http(path: "/users/{{value.userid}}", baseURL: "http://localhost:8080")
  # but not undefined values
  userUndefinedValueQuery: User
    @http(path: "/users", query: [{key: "id", value: "{{value.userid}}"}], baseURL: "http://localhost:8080")
  userVars: User @http(path: "/users/{{vars.a}}", baseURL: "http://localhost:8080")
}
```
