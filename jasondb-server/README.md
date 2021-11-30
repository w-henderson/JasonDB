# JasonDB
JasonDB is a NoSQL, document-oriented, JSON-based database management system with an emphasis on compatibility with the modern web. It supports access over TCP and WebSocket, and can handle hundreds of thousands of requests per second, making it faster than Redis on the same hardware (see the benchmarks section for more information).

JasonDB was written for a college project, so you can find a much longer and more thorough write-up on [Overleaf](https://www.overleaf.com/read/mpsrbtcksfds).

## Contents
- [Quick Start](#quick-start)
- [Query Language](#query-language)
  - [Conditions](#conditions)
- [SDK Example](#sdk-example)
  - [Async/Await](#asyncawait)
  - [Promises](#promises)
- [Benchmarks](#benchmarks)
  - [GET Performance](#get-performance)
  - [SET Performance](#set-performance)
  - [Pipelined Performance](#pipelined-performance)
- [CLI Reference](#cli-reference)

## Quick Start
To get started with JasonDB, clone the repo and run `cargo build` to compile for your system. Windows binaries are available on the [Releases](https://github.com/w-henderson/JasonDB/releases) page if you don't have Rust installed. Quickly generate an empty database by running `jasondb create <database name>`, then host this database with `jasondb <database name>`. To use WebSocket, you'll need to generate TLS keys, which you can learn more about in the TLS section. If this is not a priority, pass the `--no-ws` argument to disable WebSocket access.

## Query Language
| Command | Explanation |
| --- | --- |
| `CREATE <collection>` | Creates a collection with the given name. |
| `GET <document> FROM <collection>` | Gets the specified document from the specified collection. |
| `SET <document> FROM <collection> TO <json>` | Sets the document to the given JSON value. |
| `LIST <collection>` | Returns every document in the collection. |
| `LIST <collection> WHERE <condition>` | Queries the collection for documents which match the condition. |
| `DELETE <collection>` | Deletes the given collection. |
| `DELETE <document> FROM <collection>` | Deletes the document from the given collection. |

### Conditions
| Condition | Example | Explanation |
| --- | --- | --- |
| `<field> EQ <value>` | `country EQ France` | Matches documents in which the value of the given field equals the given value. |
| `<field> LT <value>` | `age LT 18` | Matches documents in which the value of the given field is less than the given value. |
| `<field> GT <value>` | `age GT 65` | Matches documents in which the value of the given field is greater than the given value. |

## SDK Example

### Async/Await
```js
let db = new JasonDB("localhost"); // connect to the database
let users = await db.create("users"); // get a reference to the newly created collection

await users.set("w-henderson", {name: "William Henderson"}); // set a value
let me = await users.get("w-henderson"); // retrieve the value
console.log(me.name); // "William Henderson"

let adultUsers = await users.list("age GT 17"); // get all the documents matching the condition
console.log(adultUsers);
```

### Promises
```js
let db = new JasonDB("localhost"); // connect to the database

db.collection("users") // get a promise resolving to the users collection
  .then(users => users.get("w-henderson")) // when the collection is created get the document
  .then(me => console.log(me)); // when the document has been retrieved print its value
```

## Benchmarks
As JasonDB was inspired by Redis, it makes sense to benchmark one against the other to see how it holds up. All benchmarks were performed on a Surface Pro 6 (i5), with Redis benchmarks using `redis-benchmark` and JasonDB benchmarks using a crude TCP client written in Rust. Both were performed on the local machine to minimise network latency.

### GET Performance
The GET performance was benchmarked against Redis with different numbers of connected clients. The graph below shows the requests per second (in thousands) against the number of clients for both JasonDB and Redis. It clearly shows that JasonDB consistently beats Redis, especially when there are many clients connected thanks to Rust's `Tokio` asynchronous programming interface.

<p align="center">
    <img src="https://raw.githubusercontent.com/w-henderson/JasonDB/master/assets/get_benchmark.png" width=400>
</p>

### SET Performance
The SET performance was measured in the same way, with different numbers of clients testing its multi-threaded capabilities. SET requests should, technically speaking, be slower, as the `RwLock` must be written to which can only happen from one thread at once. However, thanks to optimisations minimising the amount of instructions the thread processes when the database is locked, this hardly affects the speed. As a result of this, the database can handle a similar number of GET and SET operations per second.

<p align="center">
    <img src="https://raw.githubusercontent.com/w-henderson/JasonDB/master/assets/set_benchmark.png" width=400>
</p>

### Pipelined Performance
One of the reasons Redis is so fast is due to its ability to pipeline commands - combining multiple database operations into one network request. Through this technique, both JasonDB and Redis are able to achieve hundreds of thousands of requests per second on regular consumer hardware.

The graph below shows the number of pipelined commands per request against how many requests the database was able to handle per second, measured in thousands. It's clear that JasonDB's implementation is much faster than Redis', allowing it to reach nearly 800,000 requests per second with 100 pipelined commands. This benchmark was performed single-threaded.

<p align="center">
    <img src="https://raw.githubusercontent.com/w-henderson/JasonDB/master/assets/pipeline_benchmark.png" width=400>
</p>

## CLI Reference
```
JasonDB 0.1.2
William Henderson <william-henderson@outlook.com>
A JSON-Based Database Management System for the Web

USAGE:
    jasondb.exe [FLAGS] [OPTIONS] <DATABASE>
    jasondb.exe [FLAGS] [OPTIONS] <SUBCOMMAND>

ARGS:
    <DATABASE>    Specify the database file to load

FLAGS:
    -h, --help       Prints help information
        --no-tcp     Disable TCP listener so the database is only accessible via WebSocket
        --no-ws      Disable WebSocket listener so the database is only accessible via TCP
    -q, --quiet      Whether to suppress informative messages, recommended for large projects
    -V, --version    Prints version information

OPTIONS:
    -c, --cert <cert>            Path to TLS certificate generated by mkcert
    -i, --interval <interval>    Number of seconds between saving the database to disk, defaults to
                                 0 (continuous)
    -k, --key <key>              Key to TLS certificate generated by mkcert, defaults to "changeit"
    -l, --log <logfile>          Path to a log file
    -p, --tcp-port <tcp-port>    Port to bind the TCP listener to, defaults to 1337
    -w, --ws-port <ws-port>      Port to bind the WebSocket listener to, defaults to 1338

SUBCOMMANDS:
    create     Creates a new database with the given name
    extract    Extracts a database into a directory
    help       Prints this message or the help of the given subcommand(s)
```

## Generating TLS Keys
You need [`mkcert`](https://github.com/FiloSottile/mkcert) to generate a locally trusted key.
Run the following commands in the program directory to generate a key:
```bash
$ mkcert -install # instruct the system to trust the mkcert certificate authority
$ mkcert -pkcs12 localhost # generate a PKCS12 certificate
```
You then need to set the `.env` file configuration as follows. The key defaults to `changeit` if not changed.
```bash
CERT=<path to certificate>
KEY=<key>
```

Alternatively, you can pass the path to the certificate and the key as arguments when you run the program . For example, `jasondb myDatabase --cert <path to certificate> --key changeit`.
