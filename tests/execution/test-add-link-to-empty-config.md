# test-add-link-to-empty-config

###### check identity

#### server:

```graphql
schema @server @upstream @link(src: "../fixtures/link-const.graphql", type: Config) @link(src: "../fixtures/link-enum.graphql", type: Config) {
  query: Query
}
```
