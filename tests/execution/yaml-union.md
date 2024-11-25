# Using Union types in yaml config

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
      u:
        type:
          name: U

  NNU:
    fields:
      nu:
        type:
          name: NU
  Query:
    fields:
      test:
        type:
          name: U
        args:
          u:
            type:
              name: U
              required: true
        http:
          url: http://localhost/users/{{args.u}}/

unions:
  U:
    types: ["T1", "T2", "T3"]
```
