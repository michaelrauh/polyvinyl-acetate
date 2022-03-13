source ./build_common.sh

docker tag pvac registry.digitalocean.com/pvac-containers/pvac
docker push registry.digitalocean.com/pvac-containers/pvac
kubectl apply -f web-prod.yaml
