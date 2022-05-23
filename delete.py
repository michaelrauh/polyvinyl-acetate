#!/usr/bin/env python3

import urllib.request as r
import urllib.parse as p
import json
import time


req = r.Request(url = 'http://0.0.0.0:30001/', method = "DELETE")
r.urlopen(req)