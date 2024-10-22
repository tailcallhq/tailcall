➜  src git:(main) ✗ wrk -t4 -c100 -d10s -s big-bench.lua http://127.0.0.1:8005/graphql
Running 10s test @ http://127.0.0.1:8005/graphql
  4 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.05ms    4.15ms 130.55ms   98.78%
    Req/Sec    28.86k     8.17k  101.11k    86.00%
  1152723 requests in 10.10s, 57.01GB read
Requests/sec: 114088.16
Transfer/sec:      5.64GB
➜  src git:(main) ✗ wrk -t4 -c100 -d10s -s big-bench.lua http://127.0.0.1:8005/graphql
Running 10s test @ http://127.0.0.1:8005/graphql
  4 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.15ms    5.71ms 125.48ms   98.72%
    Req/Sec    31.99k     9.27k  108.86k    85.21%
  1272360 requests in 10.06s, 62.93GB read
Requests/sec: 126470.86
Transfer/sec:      6.26GB