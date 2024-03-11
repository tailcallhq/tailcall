# test-add-field

###### check identity


```graphql @server
schema @server @upstream(baseURL: "http://jsonplacheholder.typicode.com") {
  query: Query
}

type A {
  b: B
}

type B {
  c: String
}

type Foo {
  a: A
}

type Query @addField(name: "b", path: ["foo", "a", "b"]) {
  foo: Foo @http(path: "/foo")
}
```
