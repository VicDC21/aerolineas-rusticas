cargo run ^
      --package=server ^
      --bin=nd ^
      delete %1

timeout /t 10 /nobreak

docker container rm -f nodo-%1