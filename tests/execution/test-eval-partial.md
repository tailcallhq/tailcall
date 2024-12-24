```yaml @config
server:
  port: 8080
upstream:
  httpCache: 42
  batch:
    delay: 100
```

```graphql @schema
schema {
  query: Query
}

type Query {
  post(id: Int!): [Post] @http(url: "http://jsonplaceholder.typicode.com/posts/{{.args.id}}")
}

type User {
  id: Int!
}

type Post {
  id: Int!
  userId: Int!
  foo: String @http(url: "http://jsonplaceholder.typicode.com/posts/{{.env.FOO}}")
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/{{.value.userId}}")
}
```

```json @env
{
  "FOO": "foo"
}
```
