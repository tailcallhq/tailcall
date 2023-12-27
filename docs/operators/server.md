---
title: "@server"
sidebar_position: 1
---

The `@server` directive, when applied at the schema level, offers a comprehensive set of server configurations. It dictates how the server behaves and helps tune tailcall for various use-cases.

```graphql showLineNumbers
schema @server(...[ServerSettings]...){
    query: Query
    mutation: Mutation
}
```

In this templated structure, replace `...[ServerSettings]...` with specific configurations tailored to your project's needs. Adjust and expand these settings as necessary.

The various `ServerSettings` options and their details are explained below.

## workers

`workers` sets the number of worker threads the server will use. If not specified, the default value is the number of cores available to the system.

```graphql showLineNumbers
schema @server(workers: 32) {
  query: Query
  mutation: Mutation
}
```

In this example, the `workers` is set to `32`. This means that the Tailcall server will use 32 worker threads.

## port

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

## cacheControlHeader

The `cacheControlHeader` configuration, when activated, instructs Tailcall to transmit [Cache-Control] headers in its responses. The `max-age` value in the header, is the least of the values in the responses received by tailcall from the upstream services. By default, this is set to `false` meaning no header is set.

[cache-control]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control

```graphql showLineNumbers
schema @server(cacheControlHeader: true) {
  query: Query
  mutation: Mutation
}
```

## graphiql

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

## vars

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

## introspection

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

## queryValidation

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

## responseValidation

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

## responseHeaders

The `responseHeader` is an array of key-value pairs. These headers are added to the response of every request made to the server. This can be useful for adding headers like `Access-Control-Allow-Origin` to allow cross-origin requests, or some
additional headers like `X-Allowed-Roles` to be used by the downstream services.

```graphql showLineNumbers
schema @server(responseHeaders: [{key: "X-Allowed-Roles", value: "admin,user"}]) {
  query: Query
  mutation: Mutation
}
```

## globalResponseTimeout

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

## http

The version of HTTP to be used by the server. If not specified, the default value is `HTTP1`. The available options are `HTTP1` and `HTTP2`.

```graphql showLineNumbers
schema @server(http: HTTP2) {
  query: Query
  mutation: Mutation
}
```

## cert

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

## key

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

## batchRequests

Batching in GraphQL combines multiple requests into one, reducing server round trips.

```graphql showLineNumbers
schema @server(
  port: 8000
  batchRequests: true
)
```

### Trade-offs

Batching can improve performance but may introduce latency if one request in the batch takes longer. It also makes network traffic debugging harder.
