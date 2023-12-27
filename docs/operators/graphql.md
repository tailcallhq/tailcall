---
title: "@graphQL"
---

## @graphQL

The **@graphQL** operator allows to specify GraphQL API server request to fetch data from.

```graphql showLineNumbers
type Query {
  users: [User] @graphQL(name: "userList")
}
```

In this example, the `@graphQL` operator is used to fetch list of users from the GraphQL API upstream. The [name](#name) argument is used to specify the name of the root field on the upstream server. The inner fields from the `User` type to request are inferred from the upcoming request to the Tailcall server. The operation type of the query is inferred from the Tailcall config based on inside which operation type the `@graphQL` operator is used.

For next request with the config above:

```graphql showLineNumbers
query {
  users {
    id
    name
  }
}
```

Tailcall will request next query for the upstream:

```graphql showLineNumbers
query {
  userList {
    id
    name
  }
}
```

### baseURL

This refers to the base URL of the API. If not specified, the default base URL is the one specified in the [@upstream](#upstream) operator.

```graphql showLineNumbers
type Query {
  users: [User] @graphQL(name: "users", baseURL: "https://graphqlzero.almansi.me/api")
}
```

### name

Name of the root field on the upstream to request data from. For example:

```graphql showLineNumbers
type Query {
  users: [User] @graphQL(name: "userList")
}
```

When Tailcall receives query for `users` field it will request query for `userList` from the upstream.

### args

Named arguments for the requested field. For example:

```graphql showLineNumbers
type Query {
  user: User @graphQL(name: "user", args: [{key: "id", value: "{{value.userId}}"}])
}
```

Will request next query from the upstream for first user's name:

```graphql showLineNumbers
query {
  user(id: 1) {
    name
  }
}
```

### headers

The `headers` parameter allows you to customize the headers of the GraphQL request made by the `@graphQL` operator. It is used by specifying a key-value map of header names and their values.

For instance:

```graphql showLineNumbers
type Mutation {
  users: User @graphQL(name: "users", headers: [{key: "X-Server", value: "Tailcall"}])
}
```

In this example, a request to `/users` will include an additional HTTP header `X-Server` with the value `Tailcall`.

### batch

In case upstream GraphQL server supports request batching we can specify argument `batch` to batch several requests to single upstream into single batch request. For example:

```graphql showLineNumbers
schema @upstream(batch: {maxSize: 1000, delay: 10, headers: ["X-Server", "Authorization"]}) {
  query: Query
  mutation: Mutation
}

type Query {
  users: [User] @graphQL(name: "users", batch: true)
  posts: [Post] @graphQL(name: "posts", batch: true)
}
```

Make sure you have also specified batch settings to the `@upstream` and to the `@graphQL` operator.
