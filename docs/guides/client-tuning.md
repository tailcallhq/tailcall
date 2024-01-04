---
title: Tuning Client for Performance
---

### HTTP (Hypertext Transfer Protocol)

HTTP is like the most widely used protocol for communication between a client and a server. When you request a webpage, it's HTTP that carries your request to the server and then brings back the data to your client. HTTP is built on top of TCP.

### HTTP Versions: 1.x, 2, and 3

With each version, HTTP has become more flexible and performant.

- **HTTP/1.x**: Each HTTP request creates separate TCP connection (or a sequentially reused one).
- **HTTP/2**:
  Introduces multiplexing, allowing multiple requests and responses to be sent concurrently over a single TCP connection, improving performance.
- **HTTP/3**:
  Uses QUIC instead of TCP, further reducing connection setup time and improving handling of packet loss and network changes.

:::note
The version of the HTTP is decided by the server. So if the server only supports HTTP/1 there is no way the client can make an HTTP/2 request, even if it's compatible. However if the client only supports HTTP/1 the server as per the spec should respect and downgrade itself to serve the request over HTTP/1.
:::

### TCP (Transmission Control Protocol)

TCP is the underlying protocol that makes sure the data sent and received over the internet reaches its destination correctly and in order.

Before any data can be exchanged using HTTP, TCP establishes a connection between the client and server, like dialing a number before talking on the phone. We will see how to tune Tailcall's HTTP client to improve the performance of this connection.

:::tip
You can learn more about TCP in detail [here](https://www.techtarget.com/searchnetworking/definition/TCP).
:::

### QUIC (Quick UDP Internet Connections)

QUIC is a newer protocol developed by Google. It's designed to make web communications faster and more efficient compared to TCP. It reduces connection establishment time, is better at handling packet loss, and supports multiplexed streams over a single connection, which prevents one slow request from holding up others. It is the foundation for HTTP/3.

:::tip
You can learn more about QUIC in detail [here](https://blog.cloudflare.com/the-road-to-quic).
:::

### Why Managing Connections is Important?

- **Performance Overhead**:
  Establishing TCP connections, particularly in HTTP/1.x, can be time consuming due to the need for a complete TCP handshake for each new connection. This process adds latency and increase in system resources .

- **Limited Ports on Client Side**:
  Each TCP connection from a client requires a unique combination of an IP address and a port number. With each new connection the IP remains the same because the client is the same, however a new port is used. The number of available ports on a machine is 65535, they are shared between all the processes and not all are available for usage. So this excessive creation of new connections ultimately leads to port exhaustion on the client side, preventing it from establishing new connections and causing system failures across the processes that are running on the system.

  :::tip
  You can check out the ports to process mapping using `lsof` and `netstat` commands.
  :::

Connection pooling helps mitigate the above issues by reusing existing connections for multiple requests. This reduces the frequency of connection establishments (and thus the handshake overhead) and also conserves client-side ports. This approach enhances application performance by minimizing the resources and time spent on managing connections.

## Tuning HTTP Client

Tailcall by default uses connection pooling to manage connections and is setup with a default tuning which works well for most of the use cases. However, there are some cases where you might want to tune the HTTP client further to improve the performance of your application. Tailcall DSL provides an operator named [@upstream] which can help you to tune the HTTP client.

[@upstream]: ../operators/upstream

:::note
The connection pooling is only a meaning optimization when it comes to HTTP/1. Since HTTP/2 and HTTP/3 support multiplexing it's hard to see any observable difference in performance with pooling enabled.
:::

When using HTTP/1.x, you can tune the connection pool by using the following parameters:

### poolMaxIdlePerHost

`poolMaxIdlePerHost` is a setting that specifies the maximum number of idle connections allowed per host and defaults to `60`. Example:

```graphql showLineNumbers
schema
  @upstream(
    # highlight-start
    poolMaxIdlePerHost: 60
    # highlight-end
  ) {
  query: Query
}
```

Keeping too many idle connections can unnecessarily tie up memory and ports, while too few might lead to delays as new connections have to be established frequently. By limiting the number of idle connections, `poolMaxIdlePerHost` ensures that the system uses network and memory resources judiciously, avoiding wastage on connections that are rarely used.

If you have an application which connects to many hosts you should set this value to a lower number that way you will have connections available to connect to other hosts. On the other hand if you a few hosts and all requests have to be resolved by those hosts, you should keep a higher value for this setting.

### tcpKeepAlive

`tcpKeepAlive` is a setting that keeps TCP connections alive for the specified duration, especially during periods of inactivity. It periodically sends packets to the server to check if the connection is still open and functioning. In connection pooling, where you have a set of reusable connections, tcpKeepAlive helps in maintaining these connections in a ready-to-use state. It's particularly useful for long-lived connections in the pool. By ensuring these connections are still active, it prevents the client from attempting to use a connection that has been closed by the server due to inactivity. Without tcpKeepAlive, idle connections in the pool might get silently dropped by the server or intermediate network devices (like firewalls or load balancers). When your client tries to use such a dropped connection, it would fail, causing delays and errors. Keeping connections alive and monitored means you can efficiently reuse them, reducing the overhead of establishing new connections frequently.

Tailcall provides a parameter named `tcpKeepAlive` for the upstream which defaults to 5 seconds. Example:
schema

```graphql
@upstream (
# highlight-start
  tcpKeepAlive: 300
# highlight-end
) {
query: Query
}

```

### connectTimeout

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

In summary, the key to maximizing HTTP client performance lies in understanding the underlying protocols and thoughtful configuration of client settings through test. By doing so, developers can ensure efficient, robust, and high-performing client-server communication, essential for the smooth operation of modern web applications.
