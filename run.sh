#!/bin/bash
iperf3 -s -D
hostname=$1
echo $hostname
./monitor $hostname