helm repo add bitnami https://charts.bitnami.com/bitnami
helm install postgres-k bitnami/postgresql
docker build -t pvac .
kubectl apply -f web.yaml