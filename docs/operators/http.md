---
title: "@http"
---

This **@http** operator serves as an indication of a field or node that is underpinned by a REST API. For Example:

```graphql showLineNumbers
type Query {
  users: [User] @http(path: "/users")
}
```

In this example, the `@http` operator is added to the `users` field of the `Query` type. This means that the `users` field is underpinned by a REST API. The [path](#path) argument is used to specify the path of the REST API. In this case, the path is `/users`. This means that the GraphQL server will make a GET request to `https://jsonplaceholder.typicode.com/users` when the `users` field is queried.

## baseURL

This refers to the base URL of the API. If not specified, the default base URL is the one specified in the [@upstream](#upstream) operator.

```graphql showLineNumbers
type Query {
  users: [User] @http(path: "/users", baseURL: "https://jsonplaceholder.typicode.com")
}
```

## path

This refers to the API endpoint you're going to call. For instance https://jsonplaceholder.typicode.com/users`.

```graphql showLineNumbers
type Query {
  users: [User] @http(path: "/users")
}
```

If your API endpoint contains dynamic segments, you can use Mustache templates to substitute variables. For example, to fetch a specific user, the path can be written as `/users/{{args.id}}`.

```graphql showLineNumbers
type Query {
  user(id: ID!): User @http(path: "/users/{{args.id}}")
}
```

## method

This refers to the HTTP method of the API call. Commonly used methods include GET, POST, PUT, DELETE, etc. If not specified, the default method is GET. For example:

```graphql showLineNumbers
type Mutation {
  createUser(input: UserInput!): User @http(method: "POST", path: "/users")
}
```

## query

This represents the query parameters of your API call. You can pass it as a static object or use Mustache template for dynamic parameters. These parameters will be added to the URL. For example:

```graphql showLineNumbers
type Query {
  userPosts(id: ID!): [Post] @http(path: "/posts", query: [{key: "userId", value: "{{args.id}}"}])
}
```

## body

The body of the API call. It's used for methods like POST or PUT that send data to the server. You can pass it as a static object or use a Mustache template to substitute variables from the GraphQL variables. For example:

```graphql showLineNumbers
type Mutation {
  createUser(input: UserInput!): User @http(method: "POST", path: "/users", body: "{{args.input}}")
}
```

In the example above, the `createUser` mutation sends a POST request to `/users`, with the input object converted to JSON and included in the request body.

## headers

The `headers` parameter allows you to customize the headers of the HTTP request made by the `@http` operator. It is used by specifying a key-value map of header names and their values.

For instance:

```graphql showLineNumbers
type Mutation {
  createUser(input: UserInput!): User @http(path: "/users", headers: [{key: "X-Server", value: "Tailcall"}])
}
```

In this example, a request to `/users` will include an additional HTTP header `X-Server` with the value `Tailcall`.

You can make use of mustache templates to provide dynamic values for headers, derived from the arguments or [context] provided in the request. For example:

[context]: /docs/guides/context

```graphql showLineNumbers
type Mutation {
  users(name: String): User
    @http(path: "/users", headers: [{key: "X-Server", value: "Tailcall"}, {key: "User-Name", value: "{{args.name}}"}])
}
```

In this scenario, the `User-Name` header's value will dynamically adjust according to the `name` argument passed in the request.

## groupBy

The `groupBy` parameter groups multiple data requests into a single call. For more details please refer out [n + 1 guide].

[n + 1 guide]: /docs/guides/n+1#solving-using-batching

```graphql showLineNumbers
type Post {
  id: Int!
  name: String!
  user: User @http(path: "/users", query: [{key: "id", value: "{{value.userId}}"}], groupBy: ["id"])
}
```

- `query: {key: "id", value: "{{value.userId}}"}]`: Here, TailCall CLI is instructed to generate a URL where the user id aligns with the `userId` from the parent `Post`. For a batch of posts, the CLI compiles a single URL, such as `/users?id=1&id=2&id=3...id=10`, consolidating multiple requests into one.
