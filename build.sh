kind create cluster

helm repo add bitnami https://charts.bitnami.com/bitnami
helm install --generate-name bitnami/postgresql
