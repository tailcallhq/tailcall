---
title: "@upstream"
sidebar_position: 2
---

The `upstream` directive allows you to control various aspects of the upstream server connection. This includes settings like connection timeouts, keep-alive intervals, and more. If not specified, default values are used.

```graphql showLineNumbers
schema @upstream(...[UpstreamSetting]...){
    query: Query
    mutation: Mutation
}
```

The various `UpstreamSetting` options and their details are explained below.

## poolIdleTimeout

The time in seconds that the connection pool will wait before closing idle connections.

```graphql showLineNumbers
schema @upstream(poolIdleTimeout: 60, baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
  mutation: Mutation
}
```

## poolMaxIdlePerHost

The maximum number of idle connections that will be maintained per host.

```graphql showLineNumbers
schema @upstream(poolMaxIdlePerHost: 60, baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
  mutation: Mutation
}
```

## keepAliveInterval

The time in seconds between each keep-alive message sent to maintain the connection.

```graphql showLineNumbers
schema @upstream(keepAliveInterval: 60, baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
  mutation: Mutation
}
```

## keepAliveTimeout

The time in seconds that the connection will wait for a keep-alive message before closing.

```graphql showLineNumbers
schema @upstream(keepAliveTimeout: 60, baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
  mutation: Mutation
}
```

## keepAliveWhileIdle

A boolean value that determines whether keep-alive messages should be sent while the connection is idle.

```graphql showLineNumbers
schema @upstream(keepAliveWhileIdle: false, baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
  mutation: Mutation
}
```

## proxy

The `proxy` setting defines an intermediary server through which the upstream requests will be routed before reaching their intended endpoint. By specifying a proxy URL, you introduce an additional layer, enabling custom routing and security policies.

```graphql showLineNumbers
schema @upstream(proxy: {url: "http://localhost:3000"}, baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
  mutation: Mutation
}
```

In the provided example, we've set the proxy's `url` to "http://localhost:3000". This configuration ensures that all requests aimed at the designated `baseURL` are first channeled through this proxy. To illustrate, if the `baseURL` is "http://jsonplaceholder.typicode.com", any request targeting it would be initially sent to "http://localhost:3000" before being redirected to its final destination.

## connectTimeout

The time in seconds that the connection will wait for a response before timing out.

```graphql showLineNumbers
schema @upstream(connectTimeout: 60, baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
  mutation: Mutation
}
```

## timeout

The maximum time in seconds that the connection will wait for a response.

```graphql showLineNumbers
schema @upstream(timeout: 60, baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
  mutation: Mutation
}
```

## tcpKeepAlive

The time in seconds between each TCP keep-alive message sent to maintain the connection.

```graphql showLineNumbers
schema @upstream(tcpKeepAlive: 60, baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
  mutation: Mutation
}
```

## userAgent

The User-Agent header value to be used in HTTP requests.

```graphql showLineNumbers
schema @upstream(userAgent: "Tailcall/1.0", baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
  mutation: Mutation
}
```

## allowedHeaders

The `allowedHeaders` configuration specifies which HTTP headers are permitted to be forwarded to upstream services when making requests.
If `allowedHeaders` isn't specified, no incoming headers will be forwarded to the upstream services, which can provide an added layer of security but might restrict essential data flow.

```graphql showLineNumbers
schema @upstream(allowedHeaders: ["Authorization", "X-Api-Key"]) {
  query: Query
  mutation: Mutation
}
```

In the example above, the `allowedHeaders` is set to allow only `Authorization` and `X-Api-Key` headers. This means that requests containing these headers will forward them to upstream services, while all others will be ignored. It ensures that only expected headers are communicated to dependent services, emphasizing security and consistency.

## baseURL

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

## httpCache

When activated, directs Tailcall to utilize HTTP caching mechanisms. These mechanisms, in accordance with the [HTTP Caching RFC](https://tools.ietf.org/html/rfc7234), are designed to improve performance by reducing unnecessary data fetches. If left unspecified, this feature defaults to `false`.

```graphql showLineNumbers
schema @upstream(httpCache: false) {
  query: Query
  mutation: Mutation
}
```

### Tips

- Only use batching if necessary and other optimization techniques don't resolve performance issues.
- Use batching judiciously and monitor its impact.
- Be aware that batching can complicate debugging

## batch

An object that specifies the batch settings, including `maxSize` (the maximum size of the batch), `delay` (the delay in milliseconds between each batch), and `headers` (an array of HTTP headers to be included in the batch).

```graphql showLineNumbers
schema @upstream(batch: {maxSize: 1000, delay: 10, headers: ["X-Server", "Authorization"]}) {
  query: Query
  mutation: Mutation
}
```
