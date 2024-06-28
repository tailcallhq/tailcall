---
skip: true
---

# Test when union has too many possible types

TODO: snapshot mismatch when running the test on 32bit architecture

```graphql @config
schema @server @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Type1 {
  value: String
}

type Type2 {
  value: String
}

type Type3 {
  value: String
}

type Type4 {
  value: String
}

type Type5 {
  value: String
}

type Type6 {
  value: String
}

type Type7 {
  value: String
}

type Type8 {
  value: String
}

type Type9 {
  value: String
}

type Type10 {
  value: String
}

type Type11 {
  value: String
}

type Type12 {
  value: String
}

type Type13 {
  value: String
}

type Type14 {
  value: String
}

type Type15 {
  value: String
}

type Type16 {
  value: String
}

type Type17 {
  value: String
}

type Type18 {
  value: String
}

type Type19 {
  value: String
}

type Type20 {
  value: String
}

type Type21 {
  value: String
}

type Type22 {
  value: String
}

type Type23 {
  value: String
}

type Type24 {
  value: String
}

type Type25 {
  value: String
}

type Type26 {
  value: String
}

type Type27 {
  value: String
}

type Type28 {
  value: String
}

type Type29 {
  value: String
}

type Type30 {
  value: String
}

type Type31 {
  value: String
}

type Type32 {
  value: String
}

type Type33 {
  value: String
}

type Type34 {
  value: String
}

type Type35 {
  value: String
}

type Type36 {
  value: String
}

type Type37 {
  value: String
}

type Type38 {
  value: String
}

type Type39 {
  value: String
}

type Type40 {
  value: String
}

type Type41 {
  value: String
}

type Type42 {
  value: String
}

type Type43 {
  value: String
}

type Type44 {
  value: String
}

type Type45 {
  value: String
}

type Type46 {
  value: String
}

type Type47 {
  value: String
}

type Type48 {
  value: String
}

type Type49 {
  value: String
}

type Type50 {
  value: String
}

type Type51 {
  value: String
}

type Type52 {
  value: String
}

type Type53 {
  value: String
}

type Type54 {
  value: String
}

type Type55 {
  value: String
}

type Type56 {
  value: String
}

type Type57 {
  value: String
}

type Type58 {
  value: String
}

type Type59 {
  value: String
}

type Type60 {
  value: String
}

type Type61 {
  value: String
}

type Type62 {
  value: String
}

type Type63 {
  value: String
}

type Type64 {
  value: String
}

type Type65 {
  value: String
}

union AllTypes =
  | Type1
  | Type2
  | Type3
  | Type4
  | Type5
  | Type6
  | Type7
  | Type8
  | Type9
  | Type10
  | Type11
  | Type12
  | Type13
  | Type14
  | Type15
  | Type16
  | Type17
  | Type18
  | Type19
  | Type20
  | Type21
  | Type22
  | Type23
  | Type24
  | Type25
  | Type26
  | Type27
  | Type28
  | Type29
  | Type30
  | Type31
  | Type32
  | Type33
  | Type34
  | Type35
  | Type36
  | Type37
  | Type38
  | Type39
  | Type40
  | Type41
  | Type42
  | Type43
  | Type44
  | Type45
  | Type46
  | Type47
  | Type48
  | Type49
  | Type50
  | Type51
  | Type52
  | Type53
  | Type54
  | Type55
  | Type56
  | Type57
  | Type58
  | Type59
  | Type60
  | Type61
  | Type62
  | Type63
  | Type64
  | Type65

type Query {
  allTypes: AllTypes @http(path: "/path")
}
```
