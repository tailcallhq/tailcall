---
check_identity: true
---

# test-input-documentation

```graphql @server
schema @server @upstream(baseURL: "http://jsonplacheholder.typicode.com") {
  input: Foo
}

"""
This is a test
"""
input Foo {
  id: Int!
}
```
