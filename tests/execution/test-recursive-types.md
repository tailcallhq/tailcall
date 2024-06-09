---
error: true
---

# test-recursive-types

```graphql @config
schema @server(hostname: "0.0.0.0", port: 8000) @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Query {
  repository(owner: String!, name: String!): Repository
}

type Repository {
  id: ID!
  name: String!
  issues: [Issue]
}

type Issue {
  id: ID!
  title: String!
  repository: Repository
  relatedIssues: [Issue]
}
```
