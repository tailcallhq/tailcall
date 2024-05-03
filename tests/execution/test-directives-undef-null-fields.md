---
expect_validation_error: true
---

# test-directives-undef-null-fields

```graphql @server
schema @server(vars: [{key: "a", value: "1"}, {key: "c", value: "d"}]) {
  query: Query
}

type NestedUser {
  id: ID
}

type Post {
  id: Int!
  nestedNonScalar: User
    @http(baseURL: "http://localhost:8080", path: "/users/{{.value.nonNullableUser.nonNullableNestedUser}}")
  nestedNullable: User
    @http(baseURL: "http://localhost:8080", path: "/users/{{.value.nonNullableUser.nonNullableNestedUser.id}}")
  nestedUndefinedValue: User
    @http(baseURL: "http://localhost:8080", path: "/users/{{.value.nonNullableUser.nonNullableNestedUser.userId}}")
  nestedUserNullable: User
    @http(baseURL: "http://localhost:8080", path: "/users/{{.value.nonNullableUser.nestedUser.id}}")
  nonNullableUser: User! @http(baseURL: "http://localhost:8080", path: "/users/{{.value.id}}")
  user: User @http(baseURL: "http://localhost:8080", path: "/users/{{.value.id}}")
  userArg: User @http(baseURL: "http://localhost:8080", path: "/users/{{.args.id}}")
  userId: Int
  userInvalidDirective: User @http(baseURL: "http://localhost:8080", path: "/users/{{.Vale.userId}}")
  userNonScalar: User @http(baseURL: "http://localhost:8080", path: "/users/{{.value.nonNullableUser}}")
  userNullValue: User @http(baseURL: "http://localhost:8080", path: "/users/{{.value.userId}}")
  userNullValueQuery: User
    @http(baseURL: "http://localhost:8080", path: "/users", query: [{key: "id", value: "{{.value.id}}"}])
  userNullable: User @http(baseURL: "http://localhost:8080", path: "/users/{{.value.user.id}}")
  userUndefinedValue: User @http(baseURL: "http://localhost:8080", path: "/users/{{.value.userid}}")
  userUndefinedValueQuery: User
    @http(baseURL: "http://localhost:8080", path: "/users", query: [{key: "id", value: "{{.value.userid}}"}])
  userVars: User @http(baseURL: "http://localhost:8080", path: "/users/{{.vars.a}}")
}

type Query {
  userAccessHeadersVars(id: ID!): User
    @http(baseURL: "http://localhost:8080", path: "/user/{{.args.id}}/{{.headers.garbage}}/{{.vars.garbage}}")
  userListArg(id: [ID]): User @http(baseURL: "http://localhost:8080", path: "/user/{{.args.id}}")
  userNullableArg(id: ID): User @http(baseURL: "http://localhost:8080", path: "/user/{{.args.id}}")
  userUndefinedArg(id: ID): User @http(baseURL: "http://localhost:8080", path: "/user/{{.args.uid}}")
}

type User {
  id: ID!
  nestedUser: NestedUser
  nonNullableNestedUser: NestedUser!
}
```
