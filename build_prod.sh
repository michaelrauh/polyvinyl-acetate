source ./build_common.sh

docker tag pvac registry.digitalocean.com/pvac-containers/pvac
docker push registry.digitalocean.com/pvac-containers/pvac
kubectl apply -f web-prod.yaml

NODE_NAME=$(doctl kubernetes cluster node-pool get pvac-cluster pvac-cluster-default-pool -o json | jq -r '.[0].nodes[0].name')
WORKER_NODE_IP=$(doctl compute droplet get $NODE_NAME --template '{{.PublicIPv4}}')
echo $WORKER_NODE_IP
echo curl -X POST http://$WORKER_NODE_IP:30001/add
echo curl http://$WORKER_NODE_IP:30001/