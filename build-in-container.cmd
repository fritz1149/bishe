::建立容器，将当前路径挂载到/work，取名为builder
::docker run -it -v %cd%:/work --name builder 19231149/rust-builder /bin/bash
::以/work为工作路径，执行build命令
docker exec -w /work builder cargo build --target x86_64-unknown-linux-gnu
