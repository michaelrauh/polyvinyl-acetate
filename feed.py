#!/usr/bin/env python3

from time import sleep
import urllib.request as r
import urllib.parse as p
import json

def get(x):
	return int(r.urlopen("http://68.183.99.83:30001/" + x).read().decode('utf-8'))

def post(x, data):
    req = r.Request("http://68.183.99.83:30001/" + x)
    req.add_header('Content-Type', 'application/json')
    return r.urlopen(req, data).read().decode('utf-8')

with open("input.txt", "r") as f:
    body = f.read()

post("add/", json.dumps({'title': 'intro', 'body': body}).encode())
