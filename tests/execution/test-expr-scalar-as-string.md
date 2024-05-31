# Test expr for data that contains scalar type in string

```graphql @config
schema {
  query: Query
}

type Query {
  entry: Entry @expr(body: {num: "0", arr: "[1, 2, 3]", str: "test", obj: "{e: 1}", bool: "true"})
}

type Entry {
  num: String
  arr: String
  str: String
  obj: String
  bool: String
  nested: Nested
    @expr(
      body: {
        num: "\"{{.value.num}}\""
        arr: "{{.value.arr}}"
        str: "{{.value.str}}"
        obj: "{{.value.obj}}"
        bool: "{{.value.bool}}"
      }
    )
}

type Nested {
  num: String
  arr: String
  str: String
  obj: String
  bool: String
}
```

```yml @test
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: >
      query {
        entry {
          num
          arr
          str
          obj
          bool
          nested {
            num
            arr
            str
            obj
            bool
          }
        }
      }
```
