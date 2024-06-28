# Using Union types in yaml config

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
      u:
        type: U

  NNU:
    fields:
      nu:
        type: NU

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
          path: /users/{{args.u}}/

unions:
  U:
    types: ["T1", "T2", "T3"]
```
