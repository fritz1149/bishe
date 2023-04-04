#!/bin/bash
iperf3 -s -D
authentication=$1
echo $authentication
./monitor $authentication