# netracer
Network connections monitoring tool.

Monitors network traffic and provides interface for viewing a list of all incoming and outgoing TCP/UDP connections from a GNU\Linux server.

# Features:
- Collects data on incoming and outgoing connections via libpcap
- Stores information (ip, port) on all connections in a local database (Rust native_db)
- Web interface for viewing collected information
- Export report to Microsoft Excel (.xlsx)
- Support for TLS, Basic Auth

# Build:
Requirements: NodeJS 22.x, Rust 1.86.0
1. Web frontend
```
cd front
npm install
npm run build
```
2. Netracer executable

In the root of the repository after building front:
```
cargo build --release
```

# Usage:
```
Usage: netracer [OPTIONS] --interface <INTERFACE>

Options:
  -i, --interface <INTERFACE>
          Network interface to listen on (e.g. eth0)
  -t, --tls
          Listens to HTTPS on port tcp/3095. If not specified, then HTTP is listened to on the same port
  -c, --cert <CERT>
          For TLS - certificate file name
  -k, --key <KEY>
          For TLS - key file name
  -a, --authfile <AUTHFILE>
          Enables basic authentication by name and password. Specify the path to the file created using htpasswd
  -d, --max-dst-udp-port <MAX_DST_UDP_PORT>
          UDP Connections to destination ports above this will not be registered (for example filter out IANA private ports -d 49152) [default: 65535]
  -s, --min-src-udp-port <MIN_SRC_UDP_PORT>
          UDP Connections from source ports below this will not be registered (default 1-2048 usually server answers) [default: 2048]
  -h, --help
          Print help
  -V, --version
          Print version
```
By default UDP Connections from source ports 1-2048 not registered as usually server answers (only requests to this ports are registered),  to avoid this use ``-s 0`` flag.

To clean up database remove /var/netracer.ndb and restart application.

Application web interface is listening on 0.0.0.0:3095. Can be HTTP or HTTPS depending on command line options.

# REST API:
```
GET /con - all registered connections in json format
GET /conxls - same as /con in .xlsx format
GET /conagg - all registered connections in json format aggragated by same ports or source/destinations
GET /conaggxls - same as /conagg in .xlsx format
```
