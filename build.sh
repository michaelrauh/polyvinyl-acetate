source ./build_common.sh

helm upgrade --install ingress-nginx ingress-nginx \
  --repo https://kubernetes.github.io/ingress-nginx \
  --namespace ingress-nginx --create-namespace

kubectl apply -f cert-manager.yaml

kubectl apply -f web.yaml
kubectl apply -f relay.yaml
kubectl apply -f worker.yaml

kubectl create namespace observability
kubectl create -f jaeger-operator.yaml -n observability
sleep 20

kubectl apply -f simplest.yaml

source ./open_jaeger_port.sh