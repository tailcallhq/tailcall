# test-hostname-faliure

---

expect_validation_error: true

---

```graphql @server
schema @server(hostname: "abc") {
  query: Query
}

type User {
  id: Int
  name: String
}

type Query {
  user: User @http(path: "/users/1", baseURL: "http://jsonplaceholder.typicode.com")
}
```
