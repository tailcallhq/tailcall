# Connect RPC Curl Examples

## Using Connect Protocol

### Get All News
```bash
curl \
  --header "Content-Type: application/json" \
  --request POST \
  --data '{}' \
  http://localhost:8080/news.NewsService/GetAllNews
```

### Get Single News
```bash
curl \
  --header "Content-Type: application/json" \
  --request POST \
  --data '{"id": 1}' \
  http://localhost:8080/news.NewsService/GetNews
```

### Get Multiple News
```bash
curl \
  --header "Content-Type: application/json" \
  --request POST \
  --data '{"ids": [{"id": 1}, {"id": 2}]}' \
  http://localhost:8080/news.NewsService/GetMultipleNews
```

### Add News
```bash
curl \
  --header "Content-Type: application/json" \
  --request POST \
  --data '{
    "title": "New Article",
    "body": "This is a new article",
    "postImage": "https://via.placeholder.com/150",
    "status": "PUBLISHED"
  }' \
  http://localhost:8080/news.NewsService/AddNews
```

### Edit News
```bash
curl \
  --header "Content-Type: application/json" \
  --request POST \
  --data '{
    "id": 1,
    "title": "Updated Article",
    "body": "This article has been updated",
    "postImage": "https://via.placeholder.com/150",
    "status": "PUBLISHED"
  }' \
  http://localhost:8080/news.NewsService/EditNews
```

### Delete News
```bash
curl \
  --header "Content-Type: application/json" \
  --request POST \
  --data '{"id": 1}' \
  http://localhost:8080/news.NewsService/DeleteNews
```

## Using gRPC-Web Protocol

### Get All News (gRPC-Web)
```bash
curl \
  --header "Content-Type: application/grpc-web+json" \
  --request POST \
  --data '{}' \
  http://localhost:8080/news.NewsService/GetAllNews
```

### Get Single News (gRPC-Web)
```bash
curl \
  --header "Content-Type: application/grpc-web+json" \
  --request POST \
  --data '{"id": 1}' \
  http://localhost:8080/news.NewsService/GetNews
```

Note: The main differences between Connect and gRPC-Web protocols are:
1. Content-Type header: 
   - Connect uses: `application/json`
   - gRPC-Web uses: `application/grpc-web+json`
2. Connect protocol requires the `Connect-Protocol-Version: 1` header
3. The request/response format is slightly different, but the examples above should work for both
