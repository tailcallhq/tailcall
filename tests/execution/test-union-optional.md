# Test union optional

```graphql @schema
schema @server {
  query: Query
}

type Query {
  node: Node @expr(body: null)
  nodes: [Node]! @expr(body: [{User: {id: 1, username: "user"}}, null, {Page: {id: 2, slug: "page"}}])
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
        node {
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
