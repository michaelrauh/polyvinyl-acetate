import urllib.request as r
import urllib.parse as p
import json
import time
from helpers import *

with open ("worker_node_ip.txt") as f:
    node_ip = f.read().strip()
    
def get(x):
    return int(r.urlopen("http://" + node_ip + ":30001/" + x).read().decode('utf-8'))

def get_with_dims(dims):
    return int(r.urlopen("http://" + node_ip + ":30001/orthos?dims=" + dims).read().decode('utf-8'))

def post(x, data):
    req = r.Request("http://" + node_ip + ":30001/" + x)
    req.add_header('Content-Type', 'application/json')
    return r.urlopen(req, data).read().decode('utf-8')

def delete():
    r.urlopen(r.Request(url = "http://" + node_ip + ":30001/", method = "DELETE"))

def splat_with_dims(dims):
    return r.urlopen("http://" + node_ip + ":30001/splat?dims=" + dims).read().decode('utf-8')
