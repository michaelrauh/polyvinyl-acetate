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

assert get("count") == 0
assert get("depth") == 0

assert post("add/", json.dumps({'title': 'this is a title', 'body': 'this is a body'}).encode()) == "this is a title"

assert get("count") == 1
assert get("depth") == 0

time.sleep(1)

assert get("count") == 0
assert get("depth") == 1