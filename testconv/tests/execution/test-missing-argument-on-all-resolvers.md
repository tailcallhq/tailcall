# test-missing-argument-on-all-resolvers

###### sdl error

#### server:

```graphql
schema @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Post {
  id: Int!
}

type News {
  body: String
  id: Int
  postImage: String
  title: String
}

type NewsData {
  news: [News]!
}

type Query {
  postGraphQLArgs: Post @graphQL(name: "post", args: [{key: "id", value: "{{args.id}}"}])
  postGraphQLHeaders: Post @graphQL(name: "post", headers: [{key: "id", value: "{{args.id}}"}])
  postHttp: Post @http(path: "/posts/{{args.id}}")
  newsGrpcHeaders: NewsData!
    @grpc(
      method: "GetAllNews"
      protoPath: "tests/graphql/errors/proto/news.proto"
      service: "news.NewsService"
      headers: [{key: "id", value: "{{args.id}}"}]
    )
  newsGrpcUrl: NewsData!
    @grpc(
      method: "GetAllNews"
      protoPath: "tests/graphql/errors/proto/news.proto"
      service: "news.NewsService"
      baseURL: "{{args.url}}"
    )
  newsGrpcBody: NewsData!
    @grpc(
      method: "GetAllNews"
      protoPath: "tests/graphql/errors/proto/news.proto"
      service: "news.NewsService"
      body: "{{args.id}}"
    )
}

type User {
  id: Int
  name: String
}
```
