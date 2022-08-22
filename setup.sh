#! /bin/bash
mkdir bin
cd bin
python3 -m venv .venv
source .venv/bin/activate
cd ..
pip install -r requirements.txt
cd src/core
maturin develop
cd ..
cp -r interface/* ../bin