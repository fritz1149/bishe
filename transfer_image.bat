docker save -o dispatcher.tar 19231149/dispatcher:latest
scp dispatcher.tar fritz@192.168.137.3:/home/fritz
ssh -t fritz@192.168.137.3 "sudo docker load -i dispatcher.tar"
