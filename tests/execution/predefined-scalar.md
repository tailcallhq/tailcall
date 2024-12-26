---
error: true
---

```graphql @schema
scalar Boolean
scalar Float
scalar ID
scalar Int
scalar String
scalar DateTime

scalar Empty
scalar Email
scalar PhoneNumber
scalar Date
scalar Url
scalar JSON
scalar Int8
scalar Int16
scalar Int32
scalar Int64
scalar Int128
scalar UInt8
scalar UInt16
scalar UInt32
scalar UInt64
scalar UInt128
scalar Bytes

schema @server(port: 8000) {
  query: Query
}

type Query {
  hello: String @expr(body: "alo")
}
```
