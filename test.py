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

assert post("add/", json.dumps({'title': 'this is a title', 'body': 'this is a body. it has two sentences.'}).encode()) == "this is a title"
assert post("add/", json.dumps({'title': 'this is a different title', 'body': 'this is a body. it also has two sentences.'}).encode()) == "this is a different title"

time.sleep(5)

assert get("sentences") == 3 # the duplicate sentence will be filtered
assert get("count") == 0 # there is no pending work
assert get("depth") == 0 # the queue does not have a cycle
assert get("pairs") == 8
assert get("orthos") == 1