---
identity: true
---

# test-custom-types

```graphql @schema
schema @server @upstream {
  query: Que
  mutation: Mut
}

input PostInput {
  body: String
  title: String
  userId: Int
}

type Mut {
  insertPost(input: PostInput): Post
    @http(url: "http://jsonplaceholder.typicode.com/posts", body: "{{.args.input}}", method: "POST")
}

type Post {
  body: String
  id: Int
  title: String
  userId: Int
}

type Que {
  posts: [Post] @expr(body: [{id: 1}])
}
```
