---
title: Tuning Client for Performance
---

### HTTP (Hypertext Transfer Protocol)

HTTP is like the language used for communication between a client and a server. When you request a webpage, it's HTTP that carries your request to the server and then brings back the data to your client.

### HTTP Versions: 1.x, 2, and 3

- **HTTP/1.x**: HTTP request requires a separate TCP connection (or a sequentially reused one), which can be slow.
- **HTTP/2**:
  Introduces multiplexing, allowing multiple requests and responses to be sent concurrently over a single TCP connection, reducing latency.
- **HTTP/3**:
  Uses QUIC instead of TCP, further reducing connection setup time and improving handling of packet loss and network changes.

### TCP (Transmission Control Protocol)

TCP is the underlying protocol that makes sure the data sent and received over the internet reaches its destination correctly and in order.

Before any data can be exchanged using HTTP, TCP establishes a connection between the client and server, like dialing a number before talking on the phone. We will see how to tune Tailcall's HTTP client to improve the performance of this connection.

### QUIC (Quick UDP Internet Connections)

QUIC is a newer protocol developed by Google. It's designed to make web communications faster and more efficient compared to TCP. It reduces connection establishment time, is better at handling packet loss, and supports multiplexed streams over a single connection, which prevents one slow request from holding up others. It is the foundation for HTTP/3. It takes advantage of QUIC’s features to improve web performance.

### Why Managing Connections is Important?

- **Performance Overhead**:
  Establishing TCP connections, particularly in HTTP/1.x, can be resource-intensive due to the need for a complete TCP handshake for each new connection. This process adds latency and consumes system resources.

- **Limited Ports on Client Side**:
  Each TCP connection from a client requires a unique combination of an IP address and a port number. The number of available ports on a client machine is finite. Excessive creation of new connections can lead to port exhaustion on the client side, preventing it from establishing new connections.

- **Connection Pooling**:
  Connection pooling helps mitigate some of these issues by reusing existing connections for multiple requests. This reduces the frequency of connection establishments (and thus the handshake overhead), conserves client-side ports. This approach enhances application performance by minimizing the resources and time spent on managing connections. however, it can lead to other issues like stale connections, connection timeouts, and connection leaks. hence it is important to tune the connection pool to avoid these issues.

## Tuning HTTP Client in Tailcall

Tailcall by default uses connection pooling to manage connections and provide a default tuning which works well for most of the use cases. However, there are some cases where you might want to tune the HTTP client to improve the performance of your application for that Tailcall DSL provides a way to tune the HTTP client. Tailcall DSL provides an operator named `@upstream` which can help you to tune the HTTP client. This operator is specifically designed for tailoring the client's behavior according to your needs.

The HTTP client in Tailcall uses a connection pool to manage connections. The connection pool maintains a number of open connections to reduce the overhead of establishing a new connection for each request. Since we know HTTP/2 and HTTP/3 support multiplexing tuning the client settings might not give you much benefit. However, when using HTTP/1.x, the connection pool can be tuned to improve performance and reliability.

**Tuning for HTTP/1.x**

When using HTTP/1.x, you can tune the connection pool by using the following parameters:

**poolMaxIdlePerHost**:
`poolMaxIdlePerHost` is a setting in connection pooling that specifies the maximum number of idle connections allowed per host in the pool. This setting helps in managing the number of idle connections that are kept alive for each server or host. It's a way to balance resource usage and availability. Keeping too many idle connections can unnecessarily tie up resources, while too few might lead to delays as new connections have to be established frequently. By limiting the number of idle connections, `poolMaxIdlePerHost` ensures that the system uses network and memory resources judiciously, avoiding wastage on connections that are rarely used. If you have an application which connects to many hosts you should set this value to a lower number, otherwise, you can set it to a higher number. To get maximum performance, make sure to set it to a number that is greater than the number of concurrent requests you expect to make to a single host.

Tailcall provides a parameter named `poolMaxIdlePerHost` which can be used to set the poolMaxIdlePerHost for the HTTP client which defaults to 60. Example:

```graphql showLineNumbers
schema
  @upstream(
    # highlight-start
    poolMaxIdlePerHost: 200
    # highlight-end
  ) {
  query: Query
}
```

**tcpKeepAlive**:
`tcpKeepAlive` is a setting that keeps TCP connections alive for the specified duration, especially during periods of inactivity. It periodically sends packets to the server to check if the connection is still open and functioning. In connection pooling, where you have a set of reusable connections, tcpKeepAlive helps in maintaining these connections in a ready-to-use state. It's particularly useful for long-lived connections in the pool. By ensuring these connections are still active, it prevents the client from attempting to use a connection that has been closed by the server due to inactivity. Without tcpKeepAlive, idle connections in the pool might get silently dropped by the server or intermediate network devices (like firewalls or load balancers). When your client tries to use such a dropped connection, it would fail, causing delays and errors. Keeping connections alive and monitored means you can efficiently reuse them, reducing the overhead of establishing new connections frequently.

Tailcall provides a parameter named `tcpKeepAlive` which can be used to set the tcpKeepAlive in seconds for the HTTP client which defaults to 5 seconds. Example:

```graphql showLineNumbers
schema
  @upstream(
    # highlight-start
    tcpKeepAlive: 300
    # highlight-end
  ) {
  query: Query
}
```

**connectTimeout**:
`connectTimeout` is a specific kind of timeout that applies only to the phase where your client is trying to establish a connection with the server. When you make a connection request client tries to resolve the DNS, have SSL handshake, and establish a TCP connection. In an environment where these pods are frequently created and destroyed, it's important to have a low connectTimeout to avoid unnecessary delays. In a system using connection pooling, If a connection can't be established within the `connectTimeout` period, the attempt is aborted. This prevents the client from waiting indefinitely for a connection to be established, which could lead to delays and timeouts.

Tailcall provides a parameter named `connectTimeout` which can be used to set the connection timeout in seconds for the HTTP client which defaults to 60 seconds. Example:

```graphql showLineNumbers
schema
  @upstream(
    # highlight-start
    connectTimeout: 10
    # highlight-end
  ) {
  query: Query
}
```
