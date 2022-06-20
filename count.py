#!/usr/bin/env python3

import urllib.request as r
import urllib.parse as p
import json
import time

def get(x):
	return int(r.urlopen("http://134.122.30.203:30001/" + x).read().decode('utf-8'))

def get_with_dims(dims):
    return int(r.urlopen("http://134.122.30.203:30001/orthos?dims=" + dims).read().decode('utf-8'))


print("depth:     " + str(get("depth")))
print("sentences: " + str(get("sentences")))
print("count:     " + str(get("count")))
print("pairs:     " + str(get("pairs")))
print("phrases:   " + str(get("phrases")))
print("1,1:       " + str(get_with_dims("1,1")))
print("1,1,1:     " + str(get_with_dims("1,1,1")))
print("1,1,1,1:   " + str(get_with_dims("1,1,1,1")))
print("1,2:       " + str(get_with_dims("1,2")))
print("2,2:       " + str(get_with_dims("2,2")))
print("2,2,2:     " + str(get_with_dims("2,2,2")))