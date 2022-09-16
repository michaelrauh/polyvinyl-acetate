source ./build_common.sh

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