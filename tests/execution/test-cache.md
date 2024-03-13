# test-cache

```graphql @server
schema @server @upstream(baseURL: "http://jsonplacheholder.typicode.com") {
  query: Query
}

type Query {
  user: User @http(path: "/foo") @cache(maxAge: 300)
}

type User @cache(maxAge: 900) {
  id: Int
  name: String
}
```
