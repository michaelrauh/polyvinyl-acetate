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

time.sleep(10)
print("ingesting up by origin")

assert get("sentences") == 12 # the duplicate sentences will be filtered
assert get("count") == 0 # there is no pending work
assert get("depth") == 0 # the queue does not have a cycle
assert get("pairs") == 13 # duplicate pairs will be filtered
assert get("phrases") == 3 # many phrases will be added

# up by origin
assert get_with_dims("1,1,1") == 1 # there is one large ortho found

r.urlopen(r.Request(url = 'http://0.0.0.0:30001/', method = "DELETE"))

# up by hop
post("add/", json.dumps({'title': 'one', 'body': 'a b c d. a c. b d. a b.'}).encode())
post("add/", json.dumps({'title': 'two', 'body': 'e f. g h. e g. f h.'}).encode())
post("add/", json.dumps({'title': 'three', 'body': 'c g. d h. a e. b f.'}).encode())
time.sleep(10)
print("ingesting up by hop")
assert get_with_dims("1,1,1") == 1

r.urlopen(r.Request(url = 'http://0.0.0.0:30001/', method = "DELETE"))

# up by contents
post("add/", json.dumps({'title': 'one', 'body': 'a b c d. a c. b d. a b.'}).encode())
post("add/", json.dumps({'title': 'two', 'body': 'e f. g h. e g. f h.'}).encode())
post("add/", json.dumps({'title': 'three', 'body': 'c g. a e. b f. d h.'}).encode())
time.sleep(10)
print("ingesting up by contents")
assert get_with_dims("1,1,1") == 1

r.urlopen(r.Request(url = 'http://0.0.0.0:30001/', method = "DELETE"))

# up by ortho forward
post("add/", json.dumps({'title': 'one', 'body': 'a b c d. a c. b d. a b.'}).encode())
post("add/", json.dumps({'title': 'three', 'body': 'c g. a e. b f. d h.'}).encode())
post("add/", json.dumps({'title': 'two', 'body': 'e f. g h. e g. f h.'}).encode())

time.sleep(10)
print("ingesting up by ortho forward")
assert get_with_dims("1,1,1") == 1

r.urlopen(r.Request(url = 'http://0.0.0.0:30001/', method = "DELETE"))

# up by ortho backward
post("add/", json.dumps({'title': 'three', 'body': 'c g. a e. b f. d h.'}).encode())
post("add/", json.dumps({'title': 'two', 'body': 'e f. g h. e g. f h.'}).encode())
post("add/", json.dumps({'title': 'one', 'body': 'a b c d. a c. b d. a b.'}).encode())

time.sleep(10)
print("ingesting up by ortho backward")
assert get_with_dims("1,1,1") == 1

r.urlopen(r.Request(url = 'http://0.0.0.0:30001/', method = "DELETE"))

# a b + b e = a b e
# c d   d f   c d f
post("add/", json.dumps({'title': 'one', 'body': 'a b c d. a c. b d. a b.'}).encode()) # left ortho
post("add/", json.dumps({'title': 'two', 'body': 'b e. d f. b d. e f.'}).encode()) # right ortho
post("add/", json.dumps({'title': 'thr', 'body': 'c d f. a b e.'}).encode()) # phrases to join them with the last one being added going through the origins

time.sleep(10)
print("ingesting over by origin")

# over by origin
assert get_with_dims("2,1") == 1 # there is one wide ortho found

r.urlopen(r.Request(url = 'http://0.0.0.0:30001/', method = "DELETE"))

post("add/", json.dumps({'title': 'one', 'body': 'a b c d. a c. b d. a b.'}).encode()) # left ortho
post("add/", json.dumps({'title': 'two', 'body': 'b e. d f. b d. e f.'}).encode()) # right ortho
post("add/", json.dumps({'title': 'thr', 'body': 'a b e. c d f.'}).encode()) # phrases to join them with the last one being added going through the hops

time.sleep(10)
print("ingesting over by hop")

# over by hop
assert get_with_dims("2,1") == 1 # there is one wide ortho found

r.urlopen(r.Request(url = 'http://0.0.0.0:30001/', method = "DELETE"))


# a b c
# d e f

# d e f
# g h i

# a b c
# d e f
# g h i

post("add/", json.dumps({'title': 'one', 'body': 'a b c. d e f. a d. b e. c f'}).encode()) # left ortho
post("add/", json.dumps({'title': 'two', 'body': 'd e f. g h i. d g. e h. f i'}).encode()) # right ortho
post("add/", json.dumps({'title': 'thr', 'body': 'a d g. b e h. c f i'}).encode()) # phrases to join them with the last one being added going through the hops

time.sleep(10)
print("ingesting over by contents")

# over by contents
assert get_with_dims("2,2") == 1 # there is 3x3 ortho found

r.urlopen(r.Request(url = 'http://0.0.0.0:30001/', method = "DELETE"))

# a b + b e = a b e
# c d   d f   c d f
post("add/", json.dumps({'title': 'thr', 'body': 'c d f. a b e.'}).encode()) # phrases to join them
post("add/", json.dumps({'title': 'one', 'body': 'a b c d. a c. b d. a b.'}).encode()) # left ortho
post("add/", json.dumps({'title': 'two', 'body': 'b e. d f. b d. e f.'}).encode()) # right ortho

time.sleep(10)
print("ingesting over by ortho forward")

# over on ortho found forward
assert get_with_dims("2,1") == 1 # there is one wide ortho found

r.urlopen(r.Request(url = 'http://0.0.0.0:30001/', method = "DELETE"))

# a b + b e = a b e
# c d   d f   c d f
post("add/", json.dumps({'title': 'thr', 'body': 'c d f. a b e.'}).encode()) # phrases to join them
post("add/", json.dumps({'title': 'two', 'body': 'b e. d f. b d. e f.'}).encode()) # right ortho
post("add/", json.dumps({'title': 'one', 'body': 'a b c d. a c. b d. a b.'}).encode()) # left ortho

time.sleep(10)
print("ingesting over by ortho backward")

# over on ortho found backward
assert get_with_dims("2,1") == 1 # there is one wide ortho found

r.urlopen(r.Request(url = 'http://0.0.0.0:30001/', method = "DELETE"))