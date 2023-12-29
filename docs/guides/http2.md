---
title: Leveraging HTTP/2 with tailcall
---
HTTP/2 is a modern protocol that significantly improves web performance by introducing features like multiplexing, header compression, and more efficient connection handling. Tailcall, through its @http operator, enables seamless integration and utilization of HTTP/2 capabilities for both inbound and outbound API requests.

## Certificate and Key Generation

Before setting up HTTP/2 with Tailcall, you'll need a certificate and a key file for secure communication over HTTPS. Follow these steps to generate them

### 1.`Generate a Private Key`:
Run the following command in your terminal to generate a private key:

```bash
openssl genpkey -algorithm RSA -out key.pem -aes256
```
This command generates an encrypted RSA private key and stores it in a file named `key.pem`. You'll be prompted to set a passphrase for added security.

### 2.`Generate a Certificate Signing Request (CSR)`
Use the following command to generate a CSR:

```bash
openssl x509 -req -days 365 -in csr.pem -signkey key.pem -out cert.pem
```
This command creates a self-signed certificate (cert.pem) 

:::tip
Ensure the `cert.pem` and `key.pem` files are securely stored and accessible to your Tailcall server for HTTPS communication.
:::

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

## Handling HTTP/2 Features in Outbound Requests

Tailcall enables harnessing HTTP/2 functionalities for improved performance and efficiency

#### Query Parameters:

```graphql
type Query {
  userPosts(id: ID!): [Post] @http(path: "/posts", query: [{key: "userId", value: "{{args.id}}"}])
}
```
- Utilize query parameters with Mustache templates to dynamically construct URLs.

#### Request body:

```graphql
type Mutation {
  createUser(input: UserInput!): User @http(method: "POST", path: "/users", body: "{{args.input}}")
}
```
- For methods like POST, use the body field to include data in the request body, substituting variables using Mustache templates.

#### Custom Headers:

```graphql
type Mutation {
  users(name: String): User @http(
    path: "/users",
    headers: [
      {key: "X-Server", value: "Tailcall"},
      {key: "User-Name", value: "{{args.name}}"}
    ]
  )
}
```
- Customize headers using Mustache templates for dynamic values, derived from request arguments or context.

### Conclusion

Leveraging HTTP/2 with Tailcall empowers your application with enhanced performance, reduced latency, and efficient handling of inbound and outbound requests. Ensure proper configuration and utilization of @http directives to harness the full potential of HTTP/2 features for seamless communication with APIs.