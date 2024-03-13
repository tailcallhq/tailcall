# test-missing-root-types

---
expect_validation_error: true
---

```graphql @server
schema {
  query: QueryType
  mutation: MutationDef
}
```
