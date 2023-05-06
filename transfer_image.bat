docker save -o monitor.tar 19231149/monitor:latest
scp monitor.tar fritz@192.168.137.3:/home/fritz
ssh -t fritz@192.168.137.3 "sudo docker load -i monitor.tar"
scp monitor.tar fritz@192.168.137.4:/home/fritz
ssh -t fritz@192.168.137.4 "sudo docker load -i monitor.tar"
