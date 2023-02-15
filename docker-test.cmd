docker build -t 19231149/monitor .
docker run -v %cd%/resources:/work/resources 19231149/monitor 10.0.0.3