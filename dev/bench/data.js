window.BENCHMARK_DATA = {
  "lastUpdate": 1600085466345,
  "repoUrl": "https://github.com/MoBlaa/bot-rs-core",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "mo.blaa@pm.me",
            "name": "moblaa",
            "username": "MoBlaa"
          },
          "committer": {
            "email": "mo.blaa@pm.me",
            "name": "moblaa",
            "username": "MoBlaa"
          },
          "distinct": true,
          "id": "5b145bdb945d0c444fd3446aaa725884c1b38f53",
          "message": "using custom token",
          "timestamp": "2020-09-14T13:27:58+02:00",
          "tree_id": "5b0be2126233fdf3418eafea7b2981d9f63e0361",
          "url": "https://github.com/MoBlaa/bot-rs-core/commit/5b145bdb945d0c444fd3446aaa725884c1b38f53"
        },
        "date": 1600085465721,
        "tool": "cargo",
        "benches": [
          {
            "name": "plugin__tests__bench_call",
            "value": 178,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "plugin__tests__bench_derive_delegation",
            "value": 583,
            "range": "± 204",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_basic_scheduler",
            "value": 929,
            "range": "± 240",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_basic_scheduler_100_load",
            "value": 48004,
            "range": "± 13235",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_threaded_scheduler",
            "value": 24127,
            "range": "± 3791",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_threaded_scheduler_100_load",
            "value": 92419,
            "range": "± 22183",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_basic_scheduler",
            "value": 29064,
            "range": "± 7577",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_basic_scheduler_100_load",
            "value": 2470107,
            "range": "± 576637",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_threaded_scheduler",
            "value": 85179,
            "range": "± 17828",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_threaded_scheduler_10msload",
            "value": 4198924,
            "range": "± 1618815",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}