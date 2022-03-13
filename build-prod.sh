helm repo add bitnami https://charts.bitnami.com/bitnami
helm install postgres-k bitnami/postgresql

POSTGRES_PASSWORD=$(kubectl get secret --namespace default postgres-k-postgresql -o jsonpath="{.data.postgres-password}" | base64 --decode)
DATABASE_URL=postgres://postgres:$POSTGRES_PASSWORD@postgres-k-postgresql.default.svc.cluster.local/postgres

docker build --build-arg DATABASE_URL=$DATABASE_URL -t pvac .
docker tag pvac registry.digitalocean.com/pvac-containers/pvac
docker push registry.digitalocean.com/pvac-containers/pvac
kubectl apply -f web-prod.yaml
