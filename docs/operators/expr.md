---
title: "@expr"
---

The **@expr** operator allows composing operators with simple expressions. For example:

```graphql showLineNumbers
type Query {
  getUser(id: Int!): User
    @expr(
      body: {if: {condition: {const: {data: true}}, then: {http: {path: "/users/2"}}, else: {http: {path: "/users/1"}}}}
    )
}
```

## body

The `body` holds your expression in the form of a tree with nodes representing operations to be performed. The following nodes are supported.

### http

Equivalent to [@http](#http)

### const

Equivalent to [@const](#const)

### grpc

Equivalent to [@grpc](#grpc)

### graphQL

Equivalent to [@graphQL](#graphQL)

### if

Allows branching based on conditions
