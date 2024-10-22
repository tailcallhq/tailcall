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
---------------------------------Hardcoded data at Tailcall Server-----------
➜  src git:(main) ✗ wrk -t4 -c100 -d10s http://127.0.0.1:8005/big-json
Running 10s test @ http://127.0.0.1:8005/big-json
  4 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency   645.57us    1.55ms  66.47ms   99.04%
    Req/Sec    33.44k     5.71k   77.97k    85.29%
  1335595 requests in 10.10s, 90.94GB read
Requests/sec: 132185.24
Transfer/sec:      9.00GB
➜  src git:(main) ✗ wrk -t4 -c100 -d10s http://127.0.0.1:8005/big-json
Running 10s test @ http://127.0.0.1:8005/big-json
  4 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency   569.86us  303.95us   8.88ms   79.99%
    Req/Sec    32.64k     3.92k   74.82k    89.05%
  1306138 requests in 10.10s, 88.93GB read
Requests/sec: 129274.62
Transfer/sec:      8.80GB