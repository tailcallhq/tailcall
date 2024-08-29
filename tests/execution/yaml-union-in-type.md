# Using Union types inside usual type

```yml @config
schema:
  query: Query

types:
  T1:
    fields:
      t1:
        type:
          name: String
  T2:
    fields:
      t2:
        type:
          name: Int
  T3:
    fields:
      t3:
        type:
          name: Boolean
      t33:
        type:
          name: Float
          required: true

  NU:
    fields:
      test:
        type:
          name: String
      u:
        type:
          name: U

  NNU:
    fields:
      other:
        type:
          name: Int
      new:
        type:
          name: Boolean
      nu:
        type:
          name: NU

  Query:
    fields:
      test:
        type:
          name: U
        args:
          nu:
            type:
              name: NU
              required: true
          nnu:
            type:
              name: NNU
        http:
          baseURL: http://localhost
          path: /users/{{args.nu.u}}

unions:
  U:
    types: ["T1", "T2", "T3"]
```
