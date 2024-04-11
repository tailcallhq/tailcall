---
check_identity: false
---

# test-merge-right-with-link-config

```graphql @file:stripe-types.graphql
# Balance Schema

type Balance {
  amount: Int
  currency: String
  source_types: SourceTypes
}

type SourceTypes {
  card: Int
}

type ConnectReserved {
  amount: Int
  currency: String
}

type BalanceRoot {
  object: String
  livemode: Boolean
  pending: [Balance]
  connect_reserved: [ConnectReserved]
  available: [Balance]
}
```

```graphql @server
schema
  @server(graphiql: true)
  @upstream(allowedHeaders: ["Authorization"], baseURL: "https://api.stripe.com/v1/")
  @link(src: "stripe-types.graphql", type: Config) {
  query: Query
}

type Query {
  balance: BalanceRoot @http(path: "/balance")
}
```
