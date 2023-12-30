---
title: "@expr"
---

The **@expr** operator allows composing operators with simple expressions. For example:

```graphql showLineNumbers
type Query {
    user(id: Int!): [User] @expr(body: { http: { path: "/users/{{args.id}}"}})
}
```

## body

Can be any existing resolver: [@http](#http), [@grpc](#grpc), [@graphQL](#graphql) or [@const](#const).

The `body` object can only have one field.

