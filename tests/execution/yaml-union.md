# Using Union types in yaml config

```yml @config
schema:
  query: Query

types:
  - name: T1
    fields:
      t1:
        type:
          name: String
  - name: T2
    fields:
      t2:
        type:
          name: Int
  - name: T3
    fields:
      t3:
        type:
          name: Boolean
      t33:
        type:
          name: Float
          required: true
  - name: NU
    fields:
      u:
        type:
          name: U

  - name: NNU
    fields:
      nu:
        type:
          name: NU
  - name: Query
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
    types: [ "T1", "T2", "T3" ]
```
