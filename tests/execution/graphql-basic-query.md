```graphql @config
schema @server(port: 8000) @upstream(baseURL: "http://upstream/graphql") {
  query: Query
}

type Query {
  posts: PostsConnection @graphQL(name: "allPosts")
}

type Post implements Node {
  id: ID!
  title: String!
  body: String!
}

type PostsConnection {
  posts: [Post]
  totalCount: Int
}

interface Node {
  id: ID!
}
```

```yml @mock
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { allPosts { posts { id title } } }" }'
  response:
    status: 200
    body:
      data:
        allPosts:
          totalCount: 2
          posts:
            - id: post_1
              title: title_1
              body: body_1
            - id: post_2
              title: title_2
              body: body_2
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { posts { posts { id title } } }
```
