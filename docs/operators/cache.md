---
title: "@cache"
---

The **@cache** operator enables caching for the query, field or type it is applied to. For eg:

```graphql
schema {
  query: Query
}

type Query {
  posts: [Post] @cache(maxAge: 3000)
  user(id: Int): User
}

type Post {
  title: String
  body: String
  user: User
}

type User @cache(maxAge: 4000) {
  id: Int
  name: String @cache(maxAge: 8000)
  age: Int
}
```

## maxAge

the parameter `maxAge` takes a non-zero unsigned integer value which signifies the duration, in milliseconds, for which the value will be cached.

In the above example, the entire result of `posts` query will be cached for 3000ms. When the **@cache** operator is applied to a type, it is equivalent to applying it to each field individually. If for a type, one of the fields needs to be cached differently then this operator can be applied to that field separately and it will override the values provided for the type, as can be seen in the above example for `name` field in the `User` type.

# How does the caching work?

If **@cache** is set for a query or a field, the resolver for it will run once and the result will be stored in memory for `maxAge` milliseconds, and will expire after this duration. After the cache expires, the resolver will be run again to fetch the latest value and that value will then be cached.
