---
title: "@call"
---

The **@call** operator is used to reference an `@http` operator. It is useful when you have multiple fields that resolves from the same HTTP endpoint.

```graphql showLineNumbers
schema {
  query: Query
}

type Query {
  posts: [Post] @http(path: "/posts")
  user(id: Int!): User @http(path: "/users/{{args.id}}")
}

type User {
  id: Int!
  name: String!
  username: String!
  email: String!
}

type Post {
  id: Int!
  userId: Int!
  title: String!
  body: String!
  user: User @call(query: "user", args: [{key: "id", value: "{{value.userId}}"}])
}
```

## query

The name of the field that has the `@http` resolver to be called. It is required.

```graphql showLineNumbers
type Post {
  userId: Int!
  user: User @call(query: "user", args: [{key: "id", value: "{{value.userId}}"}])
}
```

## args

The arguments to be passed to the `@http` resolver. It is optional.

```graphql showLineNumbers
type Post {
  userId: Int!
  user: User @call(query: "user", args: [{key: "id", value: "{{value.userId}}"}])
}
```
