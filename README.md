# Event driven text folding

## Architecture
There is a single database that holds facts. There is a single queue that holds notifications of these facts being added. These are kept in sync using a relay worker that reads from an outbox and writes at least once to the queue 1000 at a time. When a fact is added, a worker from a pool will have the opportunity to use the fact and all preexisting knowledge to generate a new fact. Initial facts are posted to a web listener on the cluster.

Deployment to production is managed using digitalocean kubernetes. Local is assumed docker desktop kubernetes.

## Event Flow
1. book
    1. sentence
2. sentence
    1. pair
    2. phrase
3. pair
    1. fbbf
    2. ffbb
    3. ortho up
3. pair
    1. up by origin
    2. up by hop
    3. up by contents
1. up by origin
    1. ortho
1. up by hop
    1. ortho
1. up by contents
    1. ortho
4. fbbf
    1. ortho
4. ffbb
    1. ortho
4. phrase
    1. phrase_by_origin
    2. phrase_by_hop
    3. phrase_by_contents
1. phrase_by_origin
    1. ortho
1. phrase_by_hop
    1. ortho
1. phrase_by_contents
    1. ortho
5. ortho
    1. ortho up
    1. ortho over
1. ortho up
    1. ortho up forward
    1. ortho up backward
1. ortho over
    1. ortho over forward
    1. ortho over backward

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