# Using Union types inside usual type

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
  test: String
  u: U
}

type NNU {
  other: Int
  new: Boolean
  nu: NU
}

union U = T1 | T2 | T3

type Query {
  test(nu: NU!, nnu: NNU): U @http(url: "http://localhost/users/{{args.nu.u}}")
}
```
