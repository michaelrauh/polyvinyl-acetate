#!/usr/bin/env python3

import urllib.request as r
import urllib.parse as p
import json
import time

# a b 
# c d

# e f
# g h

def get(x):
	return int(r.urlopen("http://0.0.0.0:30001/" + x).read().decode('utf-8'))

def get_with_dims(dims):
    return int(r.urlopen("http://0.0.0.0:30001/orthos?dims=" + dims).read().decode('utf-8'))

def post(x, data):
    req = r.Request("http://0.0.0.0:30001/" + x)
    req.add_header('Content-Type', 'application/json')
    return r.urlopen(req, data).read().decode('utf-8')

r.urlopen(r.Request(url = 'http://0.0.0.0:30001/', method = "DELETE"))

post("add/", json.dumps({'title': 'one', 'body': 'a b c d. a c. b d. a b.'}).encode())
post("add/", json.dumps({'title': 'two', 'body': 'e f. g h. e g. f h.'}).encode())
post("add/", json.dumps({'title': 'three', 'body': 'b f. c g. d h. a e.'}).encode())

time.sleep(5)

assert get("sentences") == 12 # the duplicate sentences will be filtered
assert get("count") == 0 # there is no pending work
assert get("depth") == 0 # the queue does not have a cycle
assert get("pairs") == 13 # duplicate pairs will be filtered

# up by origin
assert get_with_dims("1,1,1") == 1 # there is one large ortho found

r.urlopen(r.Request(url = 'http://0.0.0.0:30001/', method = "DELETE"))

# up by hop
post("add/", json.dumps({'title': 'one', 'body': 'a b c d. a c. b d. a b.'}).encode())
post("add/", json.dumps({'title': 'two', 'body': 'e f. g h. e g. f h.'}).encode())
post("add/", json.dumps({'title': 'three', 'body': 'c g. d h. a e. b f.'}).encode())
time.sleep(5)
assert get_with_dims("1,1,1") == 1

r.urlopen(r.Request(url = 'http://0.0.0.0:30001/', method = "DELETE"))

# up by contents
post("add/", json.dumps({'title': 'one', 'body': 'a b c d. a c. b d. a b.'}).encode())
post("add/", json.dumps({'title': 'two', 'body': 'e f. g h. e g. f h.'}).encode())
post("add/", json.dumps({'title': 'three', 'body': 'c g. a e. b f. d h.'}).encode())
time.sleep(5)
assert get_with_dims("1,1,1") == 1

r.urlopen(r.Request(url = 'http://0.0.0.0:30001/', method = "DELETE"))

# up by ortho forward
post("add/", json.dumps({'title': 'one', 'body': 'a b c d. a c. b d. a b.'}).encode())
post("add/", json.dumps({'title': 'three', 'body': 'c g. a e. b f. d h.'}).encode())
post("add/", json.dumps({'title': 'two', 'body': 'e f. g h. e g. f h.'}).encode())

time.sleep(5)
assert get_with_dims("1,1,1") == 1

r.urlopen(r.Request(url = 'http://0.0.0.0:30001/', method = "DELETE"))

# up by ortho backward
post("add/", json.dumps({'title': 'three', 'body': 'c g. a e. b f. d h.'}).encode())
post("add/", json.dumps({'title': 'two', 'body': 'e f. g h. e g. f h.'}).encode())
post("add/", json.dumps({'title': 'one', 'body': 'a b c d. a c. b d. a b.'}).encode())

time.sleep(5)
assert get_with_dims("1,1,1") == 1

r.urlopen(r.Request(url = 'http://0.0.0.0:30001/', method = "DELETE"))