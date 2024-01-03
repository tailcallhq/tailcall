---
title: Entity-Level Caching using Tailcall
---

In this guide, we will explore the concept of entity-level caching in GraphQL using Tailcall and provide instructions to implement it in your GraphQL backend.

## What is Entity-Level Caching?

**Entity-level caching** in GraphQL refers to caching specific data entities without caching entire query results. Each entity is stored in a cache with a unique identifier that optimizes performance by reusing cached entities across different queries, reducing redundant data fetching and processing, especially in scenarios where certain pieces of data are frequently requested and can be reused throughout the application.

**Tailcall** leverages this caching mechanism to efficiently serve GraphQL queries, reducing latency and improving overall API performance.

## Prerequisites

1. **Tailcall Installed**: Make sure you have Tailcall installed in your GraphQL project. If not, follow the installation instructions from [here](https://tailcall.run/docs/getting_started/).

2. **GraphQL Schema**: A GraphQL schema defined for your project.

## Define GraphQL Schema and Resolvers

Create a GraphQL schema with a query, and define resolvers.

```typescript
// src/schema.ts
import {gql} from "tailcall"

export const typeDefs = gql`
  type User {
    id: ID!
    name: String!
  }

  type Query {
    user(id: ID!): User
  }
`

export const resolvers = {
  Query: {
    user: async (_: any, {id}: {id: string}) => {
      // Your resolver logic goes here
    },
  },
}
```

## Identify Cacheable Entities

Identify the entities in your GraphQL schema that are suitable for caching. These are typically data types that are frequently queried and don't change frequently.

For example, consider a `User` entity:

```graphql
type User {
  id: ID!
  name: String!
  email: String!
}
```

## Implement Entity-Level Caching

Modify your resolvers to include entity-level caching using the [`@cache`](https://tailcall.run/docs/operators/cache/) operator.

```typescript
// src/resolvers.ts
import {cache} from "tailcall"
import {User} from "./models" // Import your user model

export const resolvers = {
  Query: {
    user: cache(
      async (_: any, {id}: {id: string}) => {
        // Fetch the user from the database
        const user = await User.findById(id)

        if (!user) {
          throw new Error("User not found")
        }

        return user
      },
      {maxAge: 60000} // Cache for 60 seconds
    ),
  },
}
```

Let's say you have a `User` model representing users in your database.

```typescript
// src/models/User.ts
export class User {
  static async findById(id: string): Promise<User | null> {
    // Your database query logic to find a user by ID goes here
  }
}
```

With this setup, the `user` query resolver will be cached for 60 seconds using the `@cache` operator from Tailcall. If the same query is made within that time frame, the cached result will be returned instead of hitting the database again.

This is a basic example, and you can extend this pattern to other queries or fields that require caching in your GraphQL API. You need to make sure to adjust the caching duration (`maxAge`) based on your application's requirements.

Consider a scenario where a GraphQL query fetches a user's details and their posts. With entity-level caching in place, subsequent requests for the same user will be served from the cache, reducing the need for repeated database queries.

```graphql
query {
  getUser(userId: 123) {
    id
    name
    posts {
      id
      title
      content
    }
  }
}
```

## Test and Monitor

Thoroughly test your GraphQL API to ensure that the entity-level caching is working as expected. Monitor the cache hit rates and overall performance.
