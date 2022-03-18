source ./build_common.sh

docker tag pvac registry.digitalocean.com/pvac-containers/pvac:web
docker push registry.digitalocean.com/pvac-containers/pvac:web
kubectl apply -f web-prod.yaml

docker tag pvac-relay registry.digitalocean.com/pvac-containers/pvac:relay
docker push registry.digitalocean.com/pvac-containers/pvac:relay
kubectl apply -f relay-prod.yaml

NODE_NAME=$(doctl kubernetes cluster node-pool get pvac-cluster pvac-cluster-default-pool -o json | jq -r '.[0].nodes[0].name')
WORKER_NODE_IP=$(doctl compute droplet get $NODE_NAME --template '{{.PublicIPv4}}')

echo curl -X POST http://$WORKER_NODE_IP:30001/add -H \'Content-Type: application/json\' -d \'{\"title\": \"this is a title\", \"body\": \"this is a body\"}\'
echo curl http://$WORKER_NODE_IP:30001/
echo 

