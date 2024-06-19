# Using Union types inside usual type

```yml @config
schema:
  query: Query

types:
  T1:
    fields:
      t1:
        type: String
  T2:
    fields:
      t2:
        type: Int
  T3:
    fields:
      t3:
        type: Boolean
      t33:
        type: Float
        required: true

  NU:
    fields:
      test:
        type: String
      u:
        type: U

  NNU:
    fields:
      other:
        type: Int
      new:
        type: Boolean
      nu:
        type: NU

  Query:
    fields:
      test:
        type: U
        args:
          nu:
            type: NU
            required: true
          nnu:
            type: NNU
        http:
          baseURL: http://localhost
          path: /users/{{args.nu.u}}

unions:
  U:
    types: ["T1", "T2", "T3"]
```
