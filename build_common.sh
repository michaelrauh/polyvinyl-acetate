helm repo add bitnami https://charts.bitnami.com/bitnami
helm install postgres-k bitnami/postgresql
helm install rabbit-k bitnami/rabbitmq

POSTGRES_PASSWORD=$(kubectl get secret --namespace default postgres-k-postgresql -o jsonpath="{.data.postgres-password}" | base64 --decode)
DATABASE_URL=postgres://postgres:$POSTGRES_PASSWORD@postgres-k-postgresql.default.svc.cluster.local/postgres

RABBIT_PASSWORD=$(kubectl get secret --namespace default rabbit-k-rabbitmq -o jsonpath="{.data.rabbitmq-password}" | base64 --decode)
RABBIT_URL=amqp://user:$RABBIT_PASSWORD@rabbit-k-rabbitmq.default.svc:5672

docker build --build-arg DATABASE_URL=$DATABASE_URL --build-arg RABBIT_URL=$RABBIT_URL -t pvac .
