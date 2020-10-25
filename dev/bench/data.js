window.BENCHMARK_DATA = {
  "lastUpdate": 1603646480597,
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
      },
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
          "id": "5b3fd46f8724ab1d6f008d9380f6e6af45acc4d0",
          "message": "implemented userinfo try_from without user-id",
          "timestamp": "2020-09-14T14:28:12+02:00",
          "tree_id": "729b32281b7e45dad57fd6d10254161ff06c9b25",
          "url": "https://github.com/MoBlaa/bot-rs-core/commit/5b3fd46f8724ab1d6f008d9380f6e6af45acc4d0"
        },
        "date": 1600087013805,
        "tool": "cargo",
        "benches": [
          {
            "name": "plugin__tests__bench_call",
            "value": 181,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "plugin__tests__bench_derive_delegation",
            "value": 713,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_basic_scheduler",
            "value": 1030,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_basic_scheduler_100_load",
            "value": 53916,
            "range": "± 5591",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_threaded_scheduler",
            "value": 14799,
            "range": "± 2945",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_threaded_scheduler_100_load",
            "value": 87404,
            "range": "± 21608",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_basic_scheduler",
            "value": 31766,
            "range": "± 2755",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_basic_scheduler_100_load",
            "value": 2495108,
            "range": "± 65572",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_threaded_scheduler",
            "value": 76815,
            "range": "± 17341",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_threaded_scheduler_10msload",
            "value": 3853533,
            "range": "± 1024571",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "48cad513120884f7f1ac2c8b5b4f99974999b621",
          "message": "completed tests for userinfo from irc message",
          "timestamp": "2020-09-14T14:33:59+02:00",
          "tree_id": "61b3fdb0d548bdb72dde8c643953fc362b5ff8e4",
          "url": "https://github.com/MoBlaa/bot-rs-core/commit/48cad513120884f7f1ac2c8b5b4f99974999b621"
        },
        "date": 1600087234630,
        "tool": "cargo",
        "benches": [
          {
            "name": "plugin__tests__bench_call",
            "value": 194,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "plugin__tests__bench_derive_delegation",
            "value": 773,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_basic_scheduler",
            "value": 1074,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_basic_scheduler_100_load",
            "value": 56081,
            "range": "± 3469",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_threaded_scheduler",
            "value": 24722,
            "range": "± 5627",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_threaded_scheduler_100_load",
            "value": 102556,
            "range": "± 19092",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_basic_scheduler",
            "value": 33290,
            "range": "± 4588",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_basic_scheduler_100_load",
            "value": 2489340,
            "range": "± 189392",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_threaded_scheduler",
            "value": 86407,
            "range": "± 11356",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_threaded_scheduler_10msload",
            "value": 4405589,
            "range": "± 863924",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "409c9680862c67007713b5b33bf6971555dbcc08",
          "message": "added tests to GetUsersReq",
          "timestamp": "2020-09-14T15:13:40+02:00",
          "tree_id": "cb3c3bd647b1c38622006c961e216dfbaa606566",
          "url": "https://github.com/MoBlaa/bot-rs-core/commit/409c9680862c67007713b5b33bf6971555dbcc08"
        },
        "date": 1600089652344,
        "tool": "cargo",
        "benches": [
          {
            "name": "plugin__tests__bench_call",
            "value": 187,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "plugin__tests__bench_derive_delegation",
            "value": 695,
            "range": "± 121",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_basic_scheduler",
            "value": 1025,
            "range": "± 217",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_basic_scheduler_100_load",
            "value": 52292,
            "range": "± 10740",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_threaded_scheduler",
            "value": 23667,
            "range": "± 5504",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_threaded_scheduler_100_load",
            "value": 94138,
            "range": "± 20079",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_basic_scheduler",
            "value": 30931,
            "range": "± 6896",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_basic_scheduler_100_load",
            "value": 2218663,
            "range": "± 443840",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_threaded_scheduler",
            "value": 77640,
            "range": "± 14790",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_threaded_scheduler_10msload",
            "value": 4024961,
            "range": "± 960249",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "7f46514138ac1f2b856e1149808b3e91c4ab2e9e",
          "message": "reformatted",
          "timestamp": "2020-09-14T15:14:34+02:00",
          "tree_id": "2d21765c2012ec78922371e2a2701ba1e1624a7a",
          "url": "https://github.com/MoBlaa/bot-rs-core/commit/7f46514138ac1f2b856e1149808b3e91c4ab2e9e"
        },
        "date": 1600089695218,
        "tool": "cargo",
        "benches": [
          {
            "name": "plugin__tests__bench_call",
            "value": 161,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "plugin__tests__bench_derive_delegation",
            "value": 613,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_basic_scheduler",
            "value": 934,
            "range": "± 315",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_basic_scheduler_100_load",
            "value": 48959,
            "range": "± 21517",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_threaded_scheduler",
            "value": 14574,
            "range": "± 4028",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_threaded_scheduler_100_load",
            "value": 82423,
            "range": "± 19440",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_basic_scheduler",
            "value": 30141,
            "range": "± 6348",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_basic_scheduler_100_load",
            "value": 2214729,
            "range": "± 353030",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_threaded_scheduler",
            "value": 69562,
            "range": "± 13787",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_threaded_scheduler_10msload",
            "value": 3832440,
            "range": "± 1127577",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "c10f444b97091a926e75f98483e1d96075b2e618",
          "message": "fixed path",
          "timestamp": "2020-10-03T19:21:45+02:00",
          "tree_id": "044db27b3d4919f1a0679245649c75440abef0fa",
          "url": "https://github.com/MoBlaa/bot-rs-core/commit/c10f444b97091a926e75f98483e1d96075b2e618"
        },
        "date": 1601746117069,
        "tool": "cargo",
        "benches": [
          {
            "name": "plugin__tests__bench_call",
            "value": 185,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "plugin__tests__bench_derive_delegation",
            "value": 744,
            "range": "± 146",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_basic_scheduler",
            "value": 1048,
            "range": "± 201",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_basic_scheduler_100_load",
            "value": 52919,
            "range": "± 9651",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_threaded_scheduler",
            "value": 23371,
            "range": "± 8787",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_threaded_scheduler_100_load",
            "value": 100046,
            "range": "± 24223",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_basic_scheduler",
            "value": 32252,
            "range": "± 6233",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_basic_scheduler_100_load",
            "value": 2488405,
            "range": "± 388548",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_threaded_scheduler",
            "value": 88137,
            "range": "± 24430",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_threaded_scheduler_10msload",
            "value": 4491323,
            "range": "± 1264961",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "1503ecf1084a06c1ff01a0f332812a44f551008a",
          "message": "added common Request trait for Twitch Request structs",
          "timestamp": "2020-10-06T19:02:42+02:00",
          "tree_id": "2bda58d5b63d1901a3ac5303027c606581feb877",
          "url": "https://github.com/MoBlaa/bot-rs-core/commit/1503ecf1084a06c1ff01a0f332812a44f551008a"
        },
        "date": 1602004248407,
        "tool": "cargo",
        "benches": [
          {
            "name": "plugin__tests__bench_call",
            "value": 206,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "plugin__tests__bench_derive_delegation",
            "value": 748,
            "range": "± 205",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_basic_scheduler",
            "value": 1151,
            "range": "± 397",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_basic_scheduler_100_load",
            "value": 58916,
            "range": "± 15990",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_threaded_scheduler",
            "value": 15596,
            "range": "± 5995",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_threaded_scheduler_100_load",
            "value": 97364,
            "range": "± 51769",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_basic_scheduler",
            "value": 35196,
            "range": "± 5495",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_basic_scheduler_100_load",
            "value": 2503245,
            "range": "± 371171",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_threaded_scheduler",
            "value": 78542,
            "range": "± 18896",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_threaded_scheduler_10msload",
            "value": 4079813,
            "range": "± 1234532",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "61d33aeaa4f2c6e9be0c3efa3f6c5899b7d932e8",
          "message": "reformatted",
          "timestamp": "2020-10-25T12:24:13+01:00",
          "tree_id": "80102c3fd798481e3fb1005fca3c96ea18f79e44",
          "url": "https://github.com/MoBlaa/bot-rs-core/commit/61d33aeaa4f2c6e9be0c3efa3f6c5899b7d932e8"
        },
        "date": 1603626261310,
        "tool": "cargo",
        "benches": [
          {
            "name": "plugin__tests__bench_call",
            "value": 194,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "plugin__tests__bench_derive_delegation",
            "value": 749,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_basic_scheduler",
            "value": 1067,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_basic_scheduler_100_load",
            "value": 54265,
            "range": "± 347",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_threaded_scheduler",
            "value": 18213,
            "range": "± 5514",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_threaded_scheduler_100_load",
            "value": 102846,
            "range": "± 31927",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_basic_scheduler",
            "value": 33306,
            "range": "± 4908",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_basic_scheduler_100_load",
            "value": 2385885,
            "range": "± 11522",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_threaded_scheduler",
            "value": 83744,
            "range": "± 8465",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_threaded_scheduler_10msload",
            "value": 4805549,
            "range": "± 1565741",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "a01faea495c8f9706b1e5bd3a512dab8d74efceb",
          "message": "updated version",
          "timestamp": "2020-10-25T14:57:00+01:00",
          "tree_id": "96d10c27c788c499a5147a585fcd903b5c200d8f",
          "url": "https://github.com/MoBlaa/bot-rs-core/commit/a01faea495c8f9706b1e5bd3a512dab8d74efceb"
        },
        "date": 1603636414922,
        "tool": "cargo",
        "benches": [
          {
            "name": "plugin__tests__bench_call",
            "value": 163,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "plugin__tests__bench_derive_delegation",
            "value": 599,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_basic_scheduler",
            "value": 828,
            "range": "± 180",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_basic_scheduler_100_load",
            "value": 41219,
            "range": "± 5003",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_threaded_scheduler",
            "value": 15348,
            "range": "± 3264",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_threaded_scheduler_100_load",
            "value": 84817,
            "range": "± 16005",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_basic_scheduler",
            "value": 24757,
            "range": "± 5588",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_basic_scheduler_100_load",
            "value": 2014168,
            "range": "± 321540",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_threaded_scheduler",
            "value": 69467,
            "range": "± 17707",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_threaded_scheduler_10msload",
            "value": 3776359,
            "range": "± 1040648",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "ea74d13597dfbff54be5b9ba752a087e2cfa4808",
          "message": "fixed extracting login info in usernotice",
          "timestamp": "2020-10-25T18:15:01+01:00",
          "tree_id": "fcc78ba41eb48f61283625d30d170e21cc2ce075",
          "url": "https://github.com/MoBlaa/bot-rs-core/commit/ea74d13597dfbff54be5b9ba752a087e2cfa4808"
        },
        "date": 1603646480161,
        "tool": "cargo",
        "benches": [
          {
            "name": "plugin__tests__bench_call",
            "value": 200,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "plugin__tests__bench_derive_delegation",
            "value": 779,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_basic_scheduler",
            "value": 1141,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_basic_scheduler_100_load",
            "value": 59778,
            "range": "± 571",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_threaded_scheduler",
            "value": 17984,
            "range": "± 1284",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_1_plugin_threaded_scheduler_100_load",
            "value": 102761,
            "range": "± 13134",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_basic_scheduler",
            "value": 34146,
            "range": "± 203",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_basic_scheduler_100_load",
            "value": 2457949,
            "range": "± 8481",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_threaded_scheduler",
            "value": 81330,
            "range": "± 2258",
            "unit": "ns/iter"
          },
          {
            "name": "plugins__tests__bench_64_plugin_threaded_scheduler_10msload",
            "value": 4757452,
            "range": "± 969695",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}