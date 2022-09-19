source ./passwords.sh

helm repo add bitnami https://charts.bitnami.com/bitnami
helm install postgres-k bitnami/postgresql --set global.postgresql.auth.postgresPassword=$POSTGRES_PASSWORD
helm install rabbit-k bitnami/rabbitmq --set auth.password=$RABBIT_PASSWORD

DATABASE_URL=postgres://postgres:$POSTGRES_PASSWORD@postgres-k-postgresql.default.svc.cluster.local/postgres
RABBIT_URL=amqp://user:$RABBIT_PASSWORD@rabbit-k-rabbitmq.default.svc:5672

docker build -f Dockerfile.web --build-arg DATABASE_URL=$DATABASE_URL --build-arg RABBIT_URL=$RABBIT_URL -t pvac .
docker build -f Dockerfile.relay --build-arg DATABASE_URL=$DATABASE_URL --build-arg RABBIT_URL=$RABBIT_URL -t pvac-relay .
docker build -f Dockerfile.worker --build-arg DATABASE_URL=$DATABASE_URL --build-arg RABBIT_URL=$RABBIT_URL -t pvac-worker .

helm upgrade --install ingress-nginx ingress-nginx \
  --repo https://kubernetes.github.io/ingress-nginx \
  --namespace ingress-nginx --create-namespace

kubectl apply -f cert-manager.yaml

docker tag pvac registry.digitalocean.com/pvac-containers/pvac:web
docker push registry.digitalocean.com/pvac-containers/pvac:web
kubectl apply -f web-prod.yaml

docker tag pvac-relay registry.digitalocean.com/pvac-containers/pvac:relay
docker push registry.digitalocean.com/pvac-containers/pvac:relay
kubectl apply -f relay-prod.yaml

docker tag pvac-worker registry.digitalocean.com/pvac-containers/pvac:worker
docker push registry.digitalocean.com/pvac-containers/pvac:worker
kubectl apply -f worker-prod.yaml
kubectl scale deployment/pvac-worker --replicas=17

kubectl create namespace observability
kubectl create -f jaeger-operator.yaml -n observability
sleep 20

kubectl apply -f simplest.yaml

NODE_NAME=$(doctl kubernetes cluster node-pool get pvac-cluster pvac-cluster-default-pool -o json | jq -r '.[0].nodes[0].name')
WORKER_NODE_IP=$(doctl compute droplet get $NODE_NAME --template '{{.PublicIPv4}}')
echo curl http://$WORKER_NODE_IP:30001/
echo 

sleep 10
source ./open_jaeger_port.sh