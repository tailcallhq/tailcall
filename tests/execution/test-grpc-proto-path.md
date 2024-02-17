# test-grpc-proto-path

###### sdl error

#### server:

```graphql
schema @link(src: "tailcall/src/grpcnews.proto", type: Protobuf) {
  query: Query
}

type Query {
  news: NewsData
    @grpc(service: "news.NewsService", method: "GetAllNews", baseURL: "http://localhost:4000", protoId: "news")
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
