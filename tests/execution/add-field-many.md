---
identity: true
---

# add-field-many

```graphql @schema
schema @server @upstream {
  query: Query
}

type Foo
  @addField(name: "a", path: ["x", "a"])
  @addField(name: "b", path: ["x", "b"])
  @addField(name: "c", path: ["x", "c"]) {
  name: String
  x: X
}

type Query {
  user: Foo @http(url: "http://jsonplaceholder.typicode.com/users/1")
}

type X {
  a: String
  b: String
  c: String
}
```
