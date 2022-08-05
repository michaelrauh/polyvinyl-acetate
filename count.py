#!/usr/bin/env python3

import urllib.request as r
import urllib.parse as p
import json
import time

def get(x):
	return int(r.urlopen("http://0.0.0.0:30001/" + x).read().decode('utf-8'))

def get_with_dims(dims):
    return int(r.urlopen("http://0.0.0.0:30001/orthos?dims=" + dims).read().decode('utf-8'))

dims_11 = get_with_dims("1,1")
dims_111 = get_with_dims("1,1,1")
dims_1111 = get_with_dims("1,1,1,1")
dims_11111 = get_with_dims("1,1,1,1,1")
dims_12 = get_with_dims("1,2")
dims_112 = get_with_dims("1,1,2")
dims_1112 = get_with_dims("1,1,1,2")
dims_22 = get_with_dims("2,2")
dims_222 = get_with_dims("2,2,2")
print("depth:     " + str(get("depth")))
print("sentences: " + str(get("sentences")))
print("count:     " + str(get("count")))
print("pairs:     " + str(get("pairs")))
print("phrases:   " + str(get("phrases")))
print("1,1:       " + str(dims_11))
print("1,1,1:     " + str(dims_111))
print("1,1,1,1:   " + str(dims_1111))
print("1,1,1,1,1: " + str(dims_11111))
print("1,2:       " + str(dims_12))
print("1,1,2:     " + str(dims_112))
print("1,1,1,2:   " + str(dims_1112))
print("2,2:       " + str(dims_22))
print("2,2,2:     " + str(dims_222))

total = dims_11 + dims_111 + dims_1111 + dims_11111 + dims_12 + dims_112 + dims_1112 + dims_22 + dims_222
print("total:    ", total)