#!/usr/bin/env python3

import urllib.request as r
import urllib.parse as p
import json
import time

def get(x):
	return int(r.urlopen("http://0.0.0.0:30001/" + x).read().decode('utf-8'))

def get_with_dims(dims):
    return int(r.urlopen("http://0.0.0.0:30001/orthos?dims=" + dims).read().decode('utf-8'))


print(get("sentences"))
print(get("count"))
print(get("depth"))
print(get("pairs"))
print(get("phrases"))
print(get_with_dims("1,1"))
print(get_with_dims("1,1,1"))
