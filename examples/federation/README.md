# Apollo Federation example

1. Start tailcall subgraph examples:

- `cargo run -- start examples/apollo_federation_subgraph_post.graphql`
- `cargo run -- start examples/apollo_federation_subgraph_user.graphql`

2. Run Apollo router by one of the following methods:

- run `@apollo/gateway` with `npm start` (with `npm install` for the first time) from "examples/federation" folder
- start apollo router with `rover.sh` script (install [apollo rover](https://www.apollographql.com/docs/rover) first)

3. Navigate to `http://localhost:4000` and execute supergraph queries, see [examples](#query-examples)

# Query examples

```graphql
{
  posts {
    id
    title
    user {
      id
      name
    }
  }
}
```
