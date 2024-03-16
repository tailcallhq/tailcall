# undeclared-type-no-base-url

---

expect_validation_error: true

---

```graphql @server
schema @server {
  query: Query
}

type Query {
  users: [User] @http(path: "/users")
}
```
