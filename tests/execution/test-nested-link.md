# test-nested-link

###### check identity

#### server:

```graphql
schema @server @upstream @link(src: "fixtures/graphql-with-link.graphql", type: Config) {
  query: Query
}
```
