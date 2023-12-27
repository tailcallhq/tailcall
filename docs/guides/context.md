---
title: Context
---

In any GraphQL framework, including Tailcall, `Context` is a fundamental mechanism used for data sharing amongst various parts of your application. It is an adaptable object that is made available to every resolver in GraphQL.

## Context in Tailcall

In Tailcall, as in all GraphQL implementations, Context is a variable that is accessible to every [Operator](operators/index.md). It is used to store and access data that needs to be shared between operators.

The Context can be described using the following Typescript interface:

```typescript
interface Context {
  args: Map<string, Json>
  value: Json
  parent: Context
  env: Map<string, string>
  headers: Map<string, string>
}
```

### args

These are the arguments passed to the current query. They can be used to access the arguments of the query. For example,

```graphql showLineNumbers
type Query {
  user(id: ID!): User @http(path: "/users/{{args.id}}")
}
```

In this example, `args.id` is used to access the `id` argument passed to the `user` query.

### value

This represents the value of the current node. For instance,

```graphql showLineNumbers
type Post {
  id: ID!
  title: String!
  body: String!
  comments: [Comment] @http(path: "/posts/{{value.id}}/comments")
}
```

In the example above, `value.id` is used to access the `id` field of the `Post` type.

### parent

This denotes the context of the parent node.

```graphql showLineNumbers
type Query {
  posts: [Post] @http(path: "/posts")
}
type Post {
  id: Int!
  userId: Int!
  title: String!
  body: String!
  user: User
    @http(path: "/users", query: [{key: "id", value: "{{value.userId}}"}], matchPath: ["id"], matchKey: "userId")
}
```

In this case, `value.userId` is a way to get the `userId` information from the "parent" context of the `Post` type. Essentially, it's extracting a list or "array" of `userId` fields from multiple `Post` types. Think of `value` as a container that holds the results of a post query, with `userId` being the specific key you want to fetch from that container.

### env

This represents global environment variables for the server. This is set once when the server starts.

```graphql showLineNumbers
type Query {
  users: [User]! @http(baseUrl: "{{env.API_ENDPOINT}}", path: "/users")
}
```

In the above example, `env.API_ENDPOINT` refers to an environment variable called API_ENDPOINT, which should be defined in your server settings.

### headers

These are the headers of the request that was received by the Tailcall server.

```graphql showLineNumbers
type Query {
  commentsForUser: [Comment] @http(path: "/users/{{headers.userId}}/comments")
}
```

Here, `headers.userId` refers to a header called `userId` that should be present in the `context`. The server can use this `userId` to fetch comments for the specified user.

[operator]: /docs/intro/operators
