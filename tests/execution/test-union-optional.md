# Test union optional

```graphql @config
schema @server {
  query: Query
}

type Query {
  nodes: Node @expr(body: null)
}

union Node = User | Page

type User {
  id: ID!
  username: String!
}

type Page {
  id: ID!
  slug: String!
}
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
        nodes {
          __typename
          ... on Page {
            id
            slug
          }
          ... on User {
            id
            username
          }
        }
      }
```
