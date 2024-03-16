# test-missing-schema-query

---

expect_validation_error: true

---

```graphql @server
schema {
  mutation: Mutation
}

type Mutation {
  id: Int!
}
```
