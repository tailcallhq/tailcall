---
title: "@addField"
---

The `@addField` operator simplifies data structures and queries by adding a field that _inline_ or flattens a nested field or node within your schema. It works by modifying the schema and the data transformation process, simplifying how nested data is accessed and presented.

For instance, consider a schema:

```graphql showLineNumbers
schema {
  query: Query
}

type User @addField(name: "street", path: ["address", "street"]) {
  id: Int!
  name: String!
  username: String!
  email: String!
  phone: String
  website: String
  address: Address @modify(omit: true)
}

type Address {
  street: String!
  city: String!
  state: String!
}

type Query {
  user(id: Int!): User @http(path: "/users/{{args.id}}")
}
```

Suppose we are only interested in the `street` field in `Address`.

The `@addField` operator above, applied to the `User` type in this case, creates a field called `street` in the `User` type. It includes a `path` argument, indicating the chain of fields to be traversed from a declared field (`address` in this case), to the field within Address to be added. We can also add a `@modify(omit: true)` to omit the `address` field from the schema, since we have already made its `street` field available on the `User` type.

Post application, the schema becomes:

```graphql showLineNumbers
schema {
  query: Query
}

type User {
  id: Int!
  name: String!
  username: String!
  email: String!
  phone: String
  website: String
  street: String
}

type Query {
  user(id: Int): Post!
}
```

In the above example, since we added a `@modify(omit: true)` on the `address` field, the `Address` type is eliminated from the schema.

The `@addField` operator also take cares of nullablity of the fields. If any of the fields in the path is nullable, the resulting type will be nullable.

Additionally, `@addField` supports indexing, meaning you can specify the array index to be inlined. If a field `posts` is of type `[Post]`, and you want to, for example, get the title of the first post, you can specify the path as [`"posts"`,`"0"`,`"title"`].

```graphql showLineNumbers
type User @addField(name: "firstPostTitle", path: ["posts", "0", "title"]) {
  id: Int!
  name: String!
  username: String!
  email: String!
  phone: String
  website: String
  posts: Post @http(path: "/users/{{value.id}}/posts")
}

type Post {
  id: Int!
  userId: Int!
  title: String!
  body: String!
}
```

In conclusion, the `@addField` operator helps tidy up your schema and streamline data fetching by reducing query depth, promoting better performance and simplicity.
