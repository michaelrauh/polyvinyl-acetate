#!/usr/bin/env python3

import urllib.request as r
import urllib.parse as p
import json
import time

def get(x):
	return int(r.urlopen("http://0.0.0.0:30001/" + x).read().decode('utf-8'))

def get_with_dims():
    return int(r.urlopen("http://0.0.0.0:30001/orthos?dims=1,1,1").read().decode('utf-8'))

def post(x, data):
    req = r.Request("http://0.0.0.0:30001/" + x)
    req.add_header('Content-Type', 'application/json')
    return r.urlopen(req, data).read().decode('utf-8')

post("add/", json.dumps({'title': 'one', 'body': 'a b c d. a c. b d. a b.'}).encode())
post("add/", json.dumps({'title': 'two', 'body': 'e f. g h. e g. f h.'}).encode())
post("add/", json.dumps({'title': 'three', 'body': 'a e. b f. c g. d h.'}).encode())