#!/usr/bin/env python3

import urllib.request as r
import urllib.parse as p
import json
import time


r.urlopen(r.Request(url = 'http://0.0.0.0:30001/', method = "DELETE"))