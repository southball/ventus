```sh
LLVM_SYS_150_PREFIX=$HOME/llvm-15.0.0 cargo run --bin ventus-worker-server --release
LLVM_SYS_150_PREFIX=$HOME/llvm-15.0.0 PORT=3002 strace -f -e trace=mmap,munmap,brk cargo run --release --bin ventus-server

oha -m POST -d "Test" -n 200000 -c 8 http://localhost:3000
```