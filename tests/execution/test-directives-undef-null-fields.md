---
error: true
---

# test-directives-undef-null-fields

```yaml @config
server:
  vars: [{key: "a", value: "1"}, {key: "c", value: "d"}]
```

```graphql @schema
schema {
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
    @http(url: "http://localhost:8080/user/{{.args.id}}/{{.headers.garbage}}/{{.vars.garbage}}")
  userListArg(id: [ID]): User @http(url: "http://localhost:8080/user/{{.args.id}}")
  userNullableArg(id: ID): User @http(url: "http://localhost:8080/user/{{.args.id}}")
  userUndefinedArg(id: ID): User @http(url: "http://localhost:8080/user/{{.args.uid}}")
}

type Post {
  id: Int!
  userId: Int
  user: User @http(url: "http://localhost:8080/users/{{.value.id}}")
  nonNullableUser: User! @http(url: "http://localhost:8080/users/{{.value.id}}")
  userArg: User @http(url: "http://localhost:8080/users/{{.args.id}}")
  userInvalidDirective: User @http(url: "http://localhost:8080/users/{{.Vale.userId}}")
  userNonScalar: User @http(url: "http://localhost:8080/users/{{.value.nonNullableUser}}")
  userNullable: User @http(url: "http://localhost:8080/users/{{.value.user.id}}")
  nestedUserNullable: User @http(url: "http://localhost:8080/users/{{.value.nonNullableUser.nestedUser.id}}")
  nestedNonScalar: User @http(url: "http://localhost:8080/users/{{.value.nonNullableUser.nonNullableNestedUser}}")
  nestedUndefinedValue: User
    @http(url: "http://localhost:8080/users/{{.value.nonNullableUser.nonNullableNestedUser.userId}}")
  nestedNullable: User @http(url: "http://localhost:8080/users/{{.value.nonNullableUser.nonNullableNestedUser.id}}")
  userNullValue: User @http(url: "http://localhost:8080/users/{{.value.userId}}")
  # nullable values are allowed in queries
  userNullValueQuery: User @http(url: "http://localhost:8080/users", query: [{key: "id", value: "{{.value.id}}"}])
  userUndefinedValue: User @http(url: "http://localhost:8080/users/{{.value.userid}}")
  # but not undefined values
  userUndefinedValueQuery: User
    @http(url: "http://localhost:8080/users", query: [{key: "id", value: "{{.value.userid}}"}])
  userVars: User @http(url: "http://localhost:8080/users/{{.vars.a}}")
}
```
