#!/usr/bin/env python3

import urllib.request as r
import urllib.parse as p
import json
import time

def get_with_dims():
    return r.urlopen("http://0.0.0.0:30001/splat-all-pairs").read().decode('utf-8')


print("\n\n" + str(get_with_dims()))