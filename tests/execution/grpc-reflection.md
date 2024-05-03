# Grpc datasource

```graphql @server
ERROR [3merror[0m[2m=[0mRequest error: error sending request for url (http://localhost:50051/grpc.reflection.v1alpha.ServerReflection/ServerReflectionInfo): error trying to connect: tcp connect error: Connection refused (os error 61)
```

```yml @mock
- request:
    method: POST
    url: http://localhost:50051/grpc.reflection.v1alpha.ServerReflection/ServerReflectionInfo
    textBody: \0\0\0\0\x02:\0
  response:
    status: 200
    fileBody: grpc/reflection/news-list-services.bin

- request:
    method: POST
    url: http://localhost:50051/grpc.reflection.v1alpha.ServerReflection/ServerReflectionInfo
    textBody: \0\0\0\0\x12\"\x10news.NewsService
  response:
    status: 200
    fileBody: grpc/reflection/news-service-descriptor.bin

- request:
    method: POST
    url: http://localhost:50051/news.NewsService/GetAllNews
  response:
    status: 200
    textBody: \0\0\0\0t\n#\x08\x01\x12\x06Note 1\x1a\tContent 1\"\x0cPost image 1\n#\x08\x02\x12\x06Note 2\x1a\tContent 2\"\x0cPost image 2
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { news {news{ id }} }
```
