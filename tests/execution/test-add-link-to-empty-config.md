# test-add-link-to-empty-config

###### check identity

#### server:

```graphql
schema
  @server
  @upstream
  @link(src: "tests/graphql/fixtures/link-const.graphql", type: Config)
  @link(src: "tests/graphql/fixtures/link-enum.graphql", type: Config) {
  query: Query
}
```
