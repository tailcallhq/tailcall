---
title: Leveraging HTTP/2 with tailcall
---

HTTP/2 is a modern protocol that significantly improves web performance by introducing features like multiplexing, header compression, and more efficient connection handling. Tailcall, through its @http operator, enables seamless integration and utilization of HTTP/2 capabilities for outbound API requests.

## Ingress Side Configuration

Tailcall's @http operator allows seamless integration with HTTP/2 for server-side configurations:

Continuing from the example given in the [getting started] docs

```graphql
schema @server(port: 8000, graphiql: true, http: HTTP2, cert: "./cert.pem", key: "./key.pem") {
  query: Query
  mutation: Mutation
}
```

:::Note
Ensure the cert and key value matches with the file path
:::
[getting started]: https://tailcall.run/docs/getting_started/configuration/

- `http`: Specifies the version of HTTP to be used, where `HTTP2` indicates the utilization of the HTTP/2 protocol.
- `cert`: Points to the path of the certificate file for HTTPS. It's essential for secure communication over HTTP/2.
- `key`: Refers to the path of the key file needed for HTTP/2. It's vital for secure encryption and decryption of data.

## Egress Side Implementation

The @http operator in Tailcall provides granular control over outgoing requests, enabling efficient utilization of HTTP/2 features:

```graphql
type Query {
  users: [User] @http(path: "/users", baseURL: "https://jsonplaceholder.typicode.com")
}
```

- `path`: Specifies the API endpoint for the outgoing request.
- `baseURL`: Defines the base URL of the API. If omitted, it defaults to the @upstream operator's base URL.

In Tailcall, when you're sending requests to other services (the egress side), it figures out the best way to talk to those services all by itself. So, if the service you're talking to supports HTTP/2, Tailcall just goes ahead and uses that automatically.

### Conclusion

Leveraging HTTP/2 with Tailcall empowers your application with enhanced performance, reduced latency, and efficient handling of inbound and outbound requests. Ensure proper configuration and utilization of @http directives to harness the full potential of HTTP/2 features for seamless communication with APIs.
