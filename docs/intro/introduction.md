---
title: Problem Statement
sidebar_position: 2
slug: /
---

## Traditional API Gateway

Traditional API Gateways form the backbone of modern web based application architectures, offering a comprehensive suite of features essential for efficient API management. These gateways handle tasks such as routing, authentication, circuit breaking, caching, logging, monitoring, protocol translation and the list doesn't end!

However, API Gateways don't provide developers access to the right abstraction when it comes to configuring these capabilities. Typically a TAG would provide you with primitives that are based on the underlying protocol ie. on which the API is served. For eg: You can perform authentication, routing, rate-limiting etc. on the bases of the request headers, url or method. All of which are components of the HTTP protocol. This happens because they treat the contents of request and response bodies as mere byte sequences, without delving into their substance.

Over the years, we have gotten used to consuming and managing APIs this way. Writing our own custom abstractions and sticking it around an existing over the shelf API Gateway. Our personal experience has been that nearly all companies after a certain scale require an abstraction that's specific to their business entities and feel restricted by what the API Gateway can provide.

## Tailcall API Gateway

Based on our learnings of writing APIs at massive scale, we believe that the gateway should work around an enterprise's business entities and not the other way round. That's exactly what Tailcall helps you achieve.
Tailcall provides first-class primitives designed to interact with your business entities directly without burdening the developer with the underlying protocol. This approach grants tremendous power and flexibility, transcending protocol constraints and focusing on the nature of the API's data. Let's take a the `User` entity as an example:

```graphql
type User {
  id: ID
  name: String
  email: String
  account: Account
}

type Account {
  balance: Float
  lastUpdated: Date
}
```

`User` is a business entity that can be resolved from multiple APIs. A `/users` API could resolve the `id`, `name` & `email` and a `/accounts/:userId` could resolve the user's account `balance` and `lastUpdated`. With tailcall's API Gateway you will be able to specify just the account details as private and requiring authentication.

```graphql
type User {
  id: ID
  name: String
  email: String
  account: Account @private
}

type Account {
  balance: Float
  lastUpdated: Date
}
```

With Tailcall, specifying which parts of an entity should be public or private becomes straightforward, the platform also allows for the obfuscation of fields deemed sensitive or PII in specific contexts. This is all achievable through Tailcall's DSL, which facilitates all these complex operations efficiently and with minimal latency.

Further enhancing its capabilities, Tailcall's DSL supports sophisticated API Orchestration, going beyond mere request routing. It enables you to define the expected API structure and provides guidance on resolving each component within the entity type. For instance, consider a transaction API containing a userId. Traditionally, expanding this userId to retrieve the corresponding user details would require additional micro-services. However, with Tailcall, expressing this requirement through its DSL prompts the Tailcall runtime to automatically resolve and populate these details for you. This approach eliminates the need for any manual coding, streamlining the API management process significantly.
