# Event driven text folding

## Architecture
There is a single database that holds facts. There is a single queue that holds notifications of these facts being added. These are kept in sync using a relay worker that reads from an outbox and writes at least once to the queue. When a fact is added, a worker from a pool will have the opportunity to use the fact and all preexisting knowledge to generate a new fact. Initial facts are posted to a web listener on the cluster. In the future there may be a GUI. 

Deployment to production is managed using digitalocean kubernetes. Local is assumed docker desktop kubernetes for now. At the moment, there is no true production configuration of the database, or the option to use a managed database. Production workflows are only for test, rather than for scale, at the moment. There is no autoscaling behavior and current production configuration is 1gb single node.

## Event Flow
1. corpus added
    1. sentence added
2. sentence added
    1. pair added
    2. phrase added
3. pair added
    1. ortho added (ex-nihilo)
    2. ortho (up)
4. phrase added
    1. ortho added (over)
5. ortho added
    1. ortho (over)
    2. ortho (up)

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