helm repo add bitnami https://charts.bitnami.com/bitnami
helm install postgres-k bitnami/postgresql --set global.postgresql.auth.postgresPassword=password
docker build -t pvac .
kubectl apply -f web.yaml
