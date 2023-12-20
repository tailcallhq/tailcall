wrk.method = "POST"
wrk.body = '{"operationName":null,"variables":{},"query":"{posts{user{id}}}"}'
wrk.headers["Connection"] = "keep-alive"
wrk.headers["Content-Type"] = "application/json"