---
title: "Composition"
sidebar_position: 8
---

# Composition

Operators can be composed and used together to create new and powerful transformations.

This example illustrates the concept of composition in GraphQL, which allows you to combine multiple operations (known as "operators") to build more complex transformations of data.

The given schema is defining two data types - `User` and `Post`. The `User` type has fields `id` and `name`, and the `Post` type initially has fields `user` and `userId`.

```graphql showLineNumbers
type User {
  id: Int
  name: String
}

type Post @addField(name: "userName", path: ["user", "name"]) {
  user: User @modify(omit: true) @http(path: "/users/{{userId}}")
  userId: Int!
}
```

However, it uses a series of operators to modify the `user` field.

1. The `@addField(name: "userName", path: ["user", "name"])` operator is used to extract the `name` field from `user` and add a field called `userName` to the `Post`

2. The `@modify(omit: true)` operator is used to remove the `user` field from the final Schema.

3. The `@http(path: "/users/{{userId}}")` operator is used to instruct the resolver to make an HTTP request to fetch the user data from a specified path (i.e., `/users/{{userId}}`), where `{{userId}}` is a placeholder that would be replaced with the actual `userId` when making the request.

The schema after this transformation looks like this:

```graphql showLineNumbers
type User {
  id: Int
  name: String
}

type Post {
  userName: String
  userId: Int!
}
```

So, we've used composition of operators to take a complex object (the `User` inside the `Post`), extract a specific part of it (`name`), name that part (`userName`), and then instruct GraphQL how to fetch the data using an HTTP request.

:::info
It is important to note that the order of the operators `@modify` and `@http` doesn't matter. The resulting schema will always be the same.
:::

This is a powerful mechanism that allows you to make your GraphQL schema more precise, easier to understand, and more suitable for the specific needs of your application.
