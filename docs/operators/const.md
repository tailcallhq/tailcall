---
title: "@const"
sidebar_position: 7
---

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
