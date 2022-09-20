VOLUME_ID=$(doctl compute volume list -o json | jq -r '.[0].id')
DROPLET_ID=$(doctl compute volume list -o json | jq -r '.[0].droplet_ids[0]')
doctl compute volume-action detach --wait $VOLUME_ID $DROPLET_ID
doctl compute volume delete -f $VOLUME_ID

VOLUME_ID=$(doctl compute volume list -o json | jq -r '.[0].id')
DROPLET_ID=$(doctl compute volume list -o json | jq -r '.[0].droplet_ids[0]')
doctl compute volume-action detach --wait $VOLUME_ID $DROPLET_ID
doctl compute volume delete -f $VOLUME_ID

doctl k c d pvac-cluster -f