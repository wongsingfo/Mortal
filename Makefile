all:


rust:
	cargo build -p libriichi --lib --release
	cargo build -p libriichi --bins --no-default-features --release
	cargo build -p exe-wrapper
	cp target/release/validate_logs mortal/dataset/
	cp target/release/stat mortal/dataset/
	cp target/release/libriichi.so mortal/

pytorch-pip-cpu:
	pip install torch torchvision torchaudio --extra-index-url https://download.pytorch.org/whl/cpu
	pip install tensorboard

prepare:
	mkdir -p mortal/log/train_play
	mkdir -p mortal/log/test_play
	mkdir -p mortal/log/tensorboard
	mkdir -p mortal/model
	mkdir -p mortal/dataset

tensorboard:
	tensorboard --logdir mortal/log/tensorboard

# ls mjlog_pf4-20_n22 -1 | cut -c -31 | xargs -n 1 -I{} sh -c "./mjai-reviewer --no-review -t {} --mjai-out - | gzip > tenhou_rawlog/{}.json.gz "
