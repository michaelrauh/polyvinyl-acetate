#!/usr/bin/env python3

import urllib.request as r
import urllib.parse as p
import json
import time

def get(x):
	return int(r.urlopen("http://0.0.0.0:30001/" + x).read().decode('utf-8'))

def post(x, data):
    req = r.Request("http://0.0.0.0:30001/" + x)
    req.add_header('Content-Type', 'application/json')
    return r.urlopen(req, data).read().decode('utf-8')

assert post("add/", json.dumps({'title': 'A story about Ryan', 'body': 'Ryan coded. He coded quickly.'}).encode()) == "A story about Ryan"
assert post("add/", json.dumps({'title': 'Another story about Ryan', 'body': 'Ryan coded. Ryan debugged. He debugged quickly'}).encode()) == "Another story about Ryan"

time.sleep(5)

assert get("sentences") == 4 # the duplicate sentence will be filtered
assert get("count") == 0 # there is no pending work
assert get("depth") == 0 # the queue does not have a cycle
assert get("pairs") == 6 # duplicate pairs will be filtered
assert get("orthos") == 2
