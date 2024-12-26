# Using Union types in yaml config

```graphql @schema
schema {
  query: Query
}

type T1 {
  t1: String
}

type T2 {
  t2: Int
}

type T3 {
  t3: Boolean
  t33: Float!
}

type NU {
  u: U
}

type NNU {
  nu: NU
}

union U = T1 | T2 | T3

type Query {
  test(u: U!): U @http(url: "http://localhost/users/{{args.u}}/")
}
```
