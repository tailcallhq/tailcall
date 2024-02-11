wrk.method = "POST"
wrk.body = '{"operationName":null,"variables":{},"query":"{posts {id user { id }}}"}'
wrk.headers["Connection"] = "keep-alive"
wrk.headers["Content-Type"] = "application/json"
wrk.timeout = 10
