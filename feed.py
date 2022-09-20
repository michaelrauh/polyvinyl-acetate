#!/usr/bin/env python3

from time import sleep
import urllib.request as r
import urllib.parse as p
import json
from helpers import *

with open("input.txt", "r") as f:
    body = f.read()

post("add/", json.dumps({'title': 'intro', 'body': body}).encode())
