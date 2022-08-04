# Event driven text folding

## Architecture
There is a single database that holds facts. There is a single queue that holds notifications of these facts being added. These are kept in sync using a relay worker that reads from an outbox and writes at least once to the queue 1000 at a time. When a fact is added, a worker from a pool will have the opportunity to use the fact and all preexisting knowledge to generate a new fact. Initial facts are posted to a web listener on the cluster.

Deployment to production is managed using digitalocean kubernetes. Local is assumed docker desktop kubernetes.

## Event Flow
![Event Flow](https://user-images.githubusercontent.com/2267434/182931603-63416f40-9951-47a3-8ed3-bffd7e4a6221.png)


## Stack
1. Rust (client code)
1. Diesel (row mapper)
1. Rocket (HTTP client)
1. PostgresQL (database. Currently only Bitnami Helm)
1. Kubernetes (substrate)
1. RabbitMQ (event management. Currently only Bitnami Helm)
1. DigitalOcean (production environment)
1. Python3 (systems testing)

## Running the code
1. Enable docker desktop kubernetes
2. Run `build.sh`
3. Run `test.sh` once the cluster is up
4. Run `clean.sh` to tear down while leaving kubernetes intact

## Running in production
1. Create a docker registry in digitalocean and rename references to `pvac-containers` in all build files
1. Run `provision_prod.sh`
2. Run `build_prod.sh`
3. Run `delete_prod.sh`
4. Put text into a file called `input.txt`
5. Update the localhost location in helper files to that output by `build_prod.sh`
6. Run `./feed && watch ./count.py` to ingest and monitor
