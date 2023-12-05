---
title: Operators
sidebar_position: 1
---

Tailcall DSL builds on your existing GraphQL knowledge by allowing the addition of some custom operators. These operators provide powerful compile time guarantees to make sure your API composition is tight and robust. The operator information is used to automatically generates highly optimized resolver logic for your types.

## @server

The `@server` directive, when applied at the schema level, offers a comprehensive set of server configurations. It dictates how the server behaves and helps tune tailcall for various use-cases.

```graphql showLineNumbers
schema @server(...[ServerSettings]...){
    query: Query
    mutation: Mutation
}
```

In this templated structure, replace `...[ServerSettings]...` with specific configurations tailored to your project's needs. Adjust and expand these settings as necessary.

The various `ServerSettings` options and their details are explained below.

### workers

`workers` sets the number of worker threads the server will use. If not specified, the default value is the number of cores available to the system.

```graphql showLineNumbers
schema @server(workers: 32) {
  query: Query
  mutation: Mutation
}
```

In this example, the `workers` is set to `32`. This means that the Tailcall server will use 32 worker threads.

### port

This refers to the `port` on which the Tailcall will be running. If not specified, the default port is `8000`.

```graphql showLineNumbers
schema @server(port: 8090) {
  query: Query
  mutation: Mutation
}
```

In this example, the `port` is set to `8090`. This means that the Tailcall will be accessible at `http://localhost:8090`.

:::tip
Always lean towards non-standard ports, steering clear of typical ones like 80 or 8080. Ensure your chosen port is unoccupied.
:::

### cacheControlHeader

The `cacheControlHeader` configuration, when activated, instructs Tailcall to transmit [Cache-Control] headers in its responses. The `max-age` value in the header, is the least of the values in the responses received by tailcall from the upstream services. By default, this is set to `false` meaning no header is set.

[cache-control]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control

```graphql showLineNumbers
schema @server(cacheControlHeader: true) {
  query: Query
  mutation: Mutation
}
```

### graphiql

The `grahiql` configuration enables the GraphiQL IDE at the root (/) path within Tailcall. GraphiQL is a built-in, interactive in-browser GraphQL IDE, designed to streamline query development and testing. By default, this feature is turned off.

```graphql showLineNumbers
schema @server(port: 8000, graphiql: true) {
  query: Query
  mutation: Mutation
}
```

:::tip
While the GraphiQL interface is a powerful tool for development, it's recommended to disable it in production environments, especially if you're not exposing GraphQL APIs directly to users. This ensures an added layer of security and reduces unnecessary exposure.
:::

### vars

This configuration allows you to define local variables that can be leveraged during the server's operations. These variables are particularly handy when you need to store constant configurations, secrets, or other shared information that various operations might require.

```graphql showLineNumbers
schema @server(vars: {key: "apiKey", value: "YOUR_API_KEY_HERE"}) {
  query: Query
  mutation: Mutation
}

type Query {
  externalData: Data
    @http(path: "/external-api/data", headers: [{key: "Authorization", value: "Bearer {{vars.apiKey}}"}])
}
```

In the provided example, a variable named `apiKey` is set with a placeholder value of "YOUR_API_KEY_HERE". This configuration implies that whenever Tailcall fetches data from the `externalData` endpoint, it includes the `apiKey` in the Authorization header of the HTTP request.

:::tip
Local variables, like `apiKey`, can be instrumental in securing access to external services or providing a unified place for configurations. Ensure that sensitive information stored this way is well protected and not exposed unintentionally, especially if your Tailcall configuration is publicly accessible.
:::

### introspection

This setting governs whether introspection queries are permitted on the server. Introspection is an intrinsic feature of GraphQL, allowing clients to fetch information about the schema directly. This can be instrumental for tools and client applications to understand the types, fields, and operations available. By default, this setting is enabled (`true`).

```graphql showLineNumbers
schema @server(introspection: false) {
  query: Query
  mutation: Mutation
}
```

:::tip
Although introspection is beneficial during development and debugging stages, it's wise to consider disabling it in production environments. Turning off introspection in live deployments can enhance security by preventing potential attackers from easily discerning the schema and any associated business logic or data structures.
:::

### queryValidation

The `queryValidation` configuration specifies whether the server should validate incoming GraphQL queries against the defined schema. Validating each query ensures its conformity to the schema, preventing errors from invalid or malformed queries. However, there are situations where you might opt to disable it, notably when seeking to **enhance server performance** at the cost of such checks. This defaults to `false` if not specified.

```graphql showLineNumbers
schema @server(queryValidation: true) {
  query: Query
  mutation: Mutation
}
```

In the example above, `queryValidation` is set to `true`, enabling the validation phase for incoming queries.

:::tip
This should be enabled in dev environment to make sure the queries sent are correct and validated, however in production env, you could consider disabling it for improved performance.
:::

### responseValidation

Tailcall automatically can infer the schema of the http endpoints for you. This information can be used to validate responses that are received from the upstream services. Enabling this setting allows you to perform exactly that. If this is not specified, the default setting for `responseValidation` is `false`.

```graphql showLineNumbers
schema @server(responseValidation: true) {
  query: Query
  mutation: Mutation
}
```

:::tip
Disabling this setting will offer major performance improvements, but at the potential expense of data.
:::

### responseHeaders

The `responseHeader` is an array of key-value pairs. These headers are added to the response of every request made to the server. This can be useful for adding headers like `Access-Control-Allow-Origin` to allow cross-origin requests, or some
additional headers like `X-Allowed-Roles` to be used by the downstream services.

```graphql showLineNumbers
schema @server(responseHeaders: [{key: "X-Allowed-Roles", value: "admin,user"}]) {
  query: Query
  mutation: Mutation
}
```

### globalResponseTimeout

The `globalResponseTimeout` configuration determines the maximum duration a query is allowed to run before it's terminated by the server. Essentially, it acts as a safeguard against long-running queries that could strain resources or pose security concerns.

If not explicitly defined, there might be a system-specific or default value that applies.

```graphql showLineNumbers
schema @server(globalResponseTimeout: 5000) {
  query: Query
  mutation: Mutation
}
```

In this given example, the `globalResponseTimeout` is set to `5000` milliseconds, or 5 seconds. This means any query execution taking longer than this duration will be automatically terminated by the server.

:::tip
It's crucial to set an appropriate response timeout, especially in production environments. This not only optimizes resource utilization but also acts as a security measure against potential denial-of-service attacks where adversaries might run complex queries to exhaust server resources.
:::

### http

The version of HTTP to be used by the server. If not specified, the default value is `HTTP1`. The available options are `HTTP1` and `HTTP2`.

```graphql showLineNumbers
schema @server(http: HTTP2) {
  query: Query
  mutation: Mutation
}
```

### cert

The path to certificate(s) to be used when running the server over HTTP2 (HTTPS). If not specified, the default value is `null`.

```graphql showLineNumbers
schema @server(cert: "./cert.pem") {
  query: Query
  mutation: Mutation
}
```

<!-- prefer to use standard extension libraries -->

:::tip
The certificate can be of any extension, but it's highly recommended to use standards (`pem`, `crt`, `key`).
:::

### key

The path to key to be used when running the server over HTTP2 (HTTPS). If not specified, the default value is `null`.

```graphql showLineNumbers
schema @server(key: "./key.pem") {
  query: Query
  mutation: Mutation
}
```

:::tip
The key can be of any extension, but it's highly recommended to use standards (`pem`, `crt`, `key`).
:::

## @upstream

The `upstream` directive allows you to control various aspects of the upstream server connection. This includes settings like connection timeouts, keep-alive intervals, and more. If not specified, default values are used.

```graphql showLineNumbers
schema @upstream(...[UpstreamSetting]...){
    query: Query
    mutation: Mutation
}
```

The various `UpstreamSetting` options and their details are explained below.

### poolIdleTimeout

The time in seconds that the connection pool will wait before closing idle connections.

```graphql showLineNumbers
schema @upstream(poolIdleTimeout: 60, baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
  mutation: Mutation
}
```

### poolMaxIdlePerHost

The maximum number of idle connections that will be maintained per host.

```graphql showLineNumbers
schema @upstream(poolMaxIdlePerHost: 60, baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
  mutation: Mutation
}
```

### keepAliveInterval

The time in seconds between each keep-alive message sent to maintain the connection.

```graphql showLineNumbers
schema @upstream(keepAliveInterval: 60, baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
  mutation: Mutation
}
```

### keepAliveTimeout

The time in seconds that the connection will wait for a keep-alive message before closing.

```graphql showLineNumbers
schema @upstream(keepAliveTimeout: 60, baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
  mutation: Mutation
}
```

### keepAliveWhileIdle

A boolean value that determines whether keep-alive messages should be sent while the connection is idle.

```graphql showLineNumbers
schema @upstream(keepAliveWhileIdle: false, baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
  mutation: Mutation
}
```

### proxy

The `proxy` setting defines an intermediary server through which the upstream requests will be routed before reaching their intended endpoint. By specifying a proxy URL, you introduce an additional layer, enabling custom routing and security policies.

```graphql showLineNumbers
schema @upstream(proxy: {url: "http://localhost:3000"}, baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
  mutation: Mutation
}
```

In the provided example, we've set the proxy's `url` to "http://localhost:3000". This configuration ensures that all requests aimed at the designated `baseURL` are first channeled through this proxy. To illustrate, if the `baseURL` is "http://jsonplaceholder.typicode.com", any request targeting it would be initially sent to "http://localhost:3000" before being redirected to its final destination.

### connectTimeout

The time in seconds that the connection will wait for a response before timing out.

```graphql showLineNumbers
schema @upstream(connectTimeout: 60, baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
  mutation: Mutation
}
```

### timeout

The maximum time in seconds that the connection will wait for a response.

```graphql showLineNumbers
schema @upstream(timeout: 60, baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
  mutation: Mutation
}
```

### tcpKeepAlive

The time in seconds between each TCP keep-alive message sent to maintain the connection.

```graphql showLineNumbers
schema @upstream(tcpKeepAlive: 60, baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
  mutation: Mutation
}
```

### userAgent

The User-Agent header value to be used in HTTP requests.

```graphql showLineNumbers
schema @upstream(userAgent: "Tailcall/1.0", baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
  mutation: Mutation
}
```

### allowedHeaders

The `allowedHeaders` configuration specifies which HTTP headers are permitted to be forwarded to upstream services when making requests.
If `allowedHeaders` isn't specified, no incoming headers will be forwarded to the upstream services, which can provide an added layer of security but might restrict essential data flow.

```graphql showLineNumbers
schema @upstream(allowedHeaders: ["Authorization", "X-Api-Key"]) {
  query: Query
  mutation: Mutation
}
```

In the example above, the `allowedHeaders` is set to allow only `Authorization` and `X-Api-Key` headers. This means that requests containing these headers will forward them to upstream services, while all others will be ignored. It ensures that only expected headers are communicated to dependent services, emphasizing security and consistency.

### baseURL

This refers to the default base URL for your APIs. If it's not explicitly mentioned in the `@upstream` operator, then each [@http](#http) operator must specify its own `baseURL`. If neither `@upstream` nor [@http](#http) provides a `baseURL`, it results in a compilation error.

```graphql showLineNumbers
schema @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
  mutation: Mutation
}
```

In this representation, the `baseURL` is set as `http://jsonplaceholder.typicode.com`. Thus, all API calls made by `@http` will prepend this URL to their respective paths.

:::tip
Ensure that your base URL remains free from specific path segments.

- **GOOD:** `@upstream(baseURL: http://jsonplaceholder.typicode.com)`
- **BAD:** `@upstream(baseURL: http://jsonplaceholder.typicode.com/api)`

:::

### httpCache

When activated, directs Tailcall to utilize HTTP caching mechanisms. These mechanisms, in accordance with the [HTTP Caching RFC](https://tools.ietf.org/html/rfc7234), are designed to improve performance by reducing unnecessary data fetches. If left unspecified, this feature defaults to `false`.

```graphql showLineNumbers
schema @upstream(httpCache: false) {
  query: Query
  mutation: Mutation
}
```

### batchRequests

Batching in GraphQL combines multiple requests into one, reducing server round trips.

```graphql showLineNumbers
schema @server(
  port: 8000
  batchRequests: true
)
```

#### Trade-offs

Batching can improve performance but may introduce latency if one request in the batch takes longer. It also makes network traffic debugging harder.

#### Tips

- Only use batching if necessary and other optimization techniques don't resolve performance issues.
- Use batching judiciously and monitor its impact.
- Be aware that batching can complicate debugging

### batch

An object that specifies the batch settings, including `maxSize` (the maximum size of the batch), `delay` (the delay in milliseconds between each batch), and `headers` (an array of HTTP headers to be included in the batch).

```graphql showLineNumbers
schema @upstream(batch: {maxSize: 1000, delay: 10, headers: ["X-Server", "Authorization"]}) {
  query: Query
  mutation: Mutation
}
```

## @http

This **@http** operator serves as an indication of a field or node that is underpinned by a REST API. For Example:

```graphql showLineNumbers
type Query {
  users: [User] @http(path: "/users")
}
```

In this example, the `@http` operator is added to the `users` field of the `Query` type. This means that the `users` field is underpinned by a REST API. The [path](#path) argument is used to specify the path of the REST API. In this case, the path is `/users`. This means that the GraphQL server will make a GET request to `https://jsonplaceholder.typicode.com/users` when the `users` field is queried.

### baseURL

This refers to the base URL of the API. If not specified, the default base URL is the one specified in the [@upstream](#upstream) operator.

```graphql showLineNumbers
type Query {
  users: [User] @http(path: "/users", baseURL: "https://jsonplaceholder.typicode.com")
}
```

### path

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

### method

This refers to the HTTP method of the API call. Commonly used methods include GET, POST, PUT, DELETE, etc. If not specified, the default method is GET. For example:

```graphql showLineNumbers
type Mutation {
  createUser(input: UserInput!): User @http(method: "POST", path: "/users")
}
```

### query

This represents the query parameters of your API call. You can pass it as a static object or use Mustache template for dynamic parameters. These parameters will be added to the URL. For example:

```graphql showLineNumbers
type Query {
  userPosts(id: ID!): [Post] @http(path: "/posts", query: [{key: "userId", value: "{{args.id}}"}])
}
```

### body

The body of the API call. It's used for methods like POST or PUT that send data to the server. You can pass it as a static object or use a Mustache template to substitute variables from the GraphQL variables. For example:

```graphql showLineNumbers
type Mutation {
  createUser(input: UserInput!): User @http(method: "POST", path: "/users", body: "{{args.input}}")
}
```

In the example above, the `createUser` mutation sends a POST request to `/users`, with the input object converted to JSON and included in the request body.

### headers

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

### groupBy

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

## @modify

The `@modify` operator in GraphQL provides the flexibility to alter the attributes of a field or a node within your GraphQL schema. Here's how you can use this operator:

### name

You can rename a field or a node in your GraphQL schema using the `name` argument in the `@modify` operator. This can be helpful when the field name in your underlying data source doesn't match the desired field name in your schema. For instance:

```graphql showLineNumbers
type User {
  id: Int! @modify(name: "userId")
}
```

`@modify(name: "userId")` tells GraphQL that although the field is referred to as `id`in the underlying data source, it should be presented as `userId` in your schema.

### omit

You can exclude a field or a node from your GraphQL schema using the `omit` argument in the `@modify` operator. This can be useful if you want to keep certain data hidden from the client. For instance:

```graphql showLineNumbers
type User {
  id: Int! @modify(omit: true)
}
```

`@modify(omit: true)` tells GraphQL that the `id` field should not be included in the schema, thus it won't be accessible to the client.

## @addField

The `@addField` operator simplifies data structures and queries by adding a field that _inlines_ or flattens a nested field or node within your schema. It works by modifying the schema and the data transformation process, simplifying how nested data is accessed and presented.

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

## @const

The `@const` operators allows us to embed a constant response for the schema. For eg:

```graphql
schema {
  query: Query
}

type Query {
  user: User @const(data: {name: "John", age: 12})
}

type User {
  name: String
  age: Int
}
```

The const operator will also validate the provided value at compile time to make sure that it matches the of the field. If the schema of the provided value doesn't match the type of the field, a descriptive error message is show on the console.
