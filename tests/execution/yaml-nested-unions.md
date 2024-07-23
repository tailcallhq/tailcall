# Using union types inside other union types

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

  T4:
    fields:
      t4:
        type: String

  T5:
    fields:
      t5:
        type: Boolean

  Query:
    fields:
      test:
        type: U
        args:
          u:
            type: U
            required: true
        http:
          baseURL: http://localhost
          path: /users/{{args.u}}

unions:
  U1:
    types: ["T1", "T2", "T3"]
  U2:
    types: ["T3", "T4"]
  U:
    types: ["U1", "U2", "T5"]
```
