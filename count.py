#!/usr/bin/env python3

import urllib.request as r
import urllib.parse as p
import json
import time
from helpers import *

dims_11 = get_with_dims("1,1")
dims_111 = get_with_dims("1,1,1")
dims_1111 = get_with_dims("1,1,1,1")
dims_11111 = get_with_dims("1,1,1,1,1")
dims_12 = get_with_dims("1,2")
dims_112 = get_with_dims("1,1,2")
dims_1112 = get_with_dims("1,1,1,2")
dims_22 = get_with_dims("2,2")
dims_222 = get_with_dims("2,2,2")
depth = get("depth")
sentences = get("sentences")
pairs = get("pairs")
phrases = get("phrases")
print("depth:      " + str(depth))
print("sentences:  " + str(sentences))
print("count:      " + str(get("count")))
print("pairs:      " + str(pairs))
print("phrases:    " + str(phrases))
print("1,1:        " + str(dims_11))
print("1,1,1:      " + str(dims_111))
print("1,1,1,1:    " + str(dims_1111))
print("1,1,1,1,1:  " + str(dims_11111))
print("1,2:        " + str(dims_12))
print("1,1,2:      " + str(dims_112))
print("1,1,1,2:    " + str(dims_1112))
print("2,2:        " + str(dims_22))
print("2,2,2:      " + str(dims_222))

total = dims_11 + dims_111 + dims_1111 + dims_11111 + dims_12 + dims_112 + dims_1112 + dims_22 + dims_222
everything = total + sentences + pairs + phrases
all_things = everything + depth
print("total:     ", total)
print("everything:", everything)
print("all things:", all_things)