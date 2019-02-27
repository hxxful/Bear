#!/bin/bash
sudo apt install gcc-multilib

rm build32 -rf
rm build64 -rf
mkdir build32 build64

sudo rm /opt/bear -rf
sudo mkdir /opt/bear
sudo mkdir -p /opt/bear/lib/x86_64-linux-gnu/
sudo mkdir -p /opt/bear/lib/i386-linux-gnu/
sudo mkdir -p /opt/bear/bin


cd build32
cmake .. -DCMAKE_C_COMPILER_ARG1="-m32"; make all;
cd ../build64
cmake .. -DCMAKE_C_COMPILER_ARG1="-m64"; make all;
cmake .. -DCMAKE_C_COMPILER_ARG1="-m64" -DDEFAULT_PRELOAD_FILE='/opt/bear/$LIB/libear.so'; make all; 
cd ..
sudo install -m 0644 build32/libear/libear.so /opt/bear/lib/i386-linux-gnu/libear.so
sudo install -m 0644 build64/libear/libear.so /opt/bear/lib/x86_64-linux-gnu/libear.so
sudo install -m 0555 build64/bear/bear /opt/bear/bin/bear
echo "export PATH=\$PATH:/opt/bear/bin" >> ~/.bashrc

