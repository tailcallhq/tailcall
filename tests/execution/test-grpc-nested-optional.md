# test-grpc-nested-optional

###### sdl error

#### server:

```graphql
schema @link(id: "news", src: "../graphql/errors/proto/news.proto", type: Protobuf) {
  query: Query
}

type Query {
  news: NewsData! @grpc(method: "news.NewsService.GetAllNews", baseURL: "http://localhost:4000")
}

type NewsData {
  news: [News]
}

type News {
  id: Int!
  title: String!
  body: String!
  postImage: String!
}
```
