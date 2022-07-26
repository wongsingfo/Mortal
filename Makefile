all:

online:
	cp mortal/config-online.toml mortal/config.toml
	cp mortal/model/grp-v0.pth mortal/model/grp.pth
	(cd mortal && python train.py)

online-server:
	cp mortal/config-offline.toml mortal/config.toml
	(cd mortal && python server.py)

online-client:
	cp mortal/config-offline.toml mortal/config.toml
	(cd mortal && python client.py)

1v3:
	cp mortal/config-offline.toml mortal/config.toml
	(cd mortal && python one_vs_three.py)

grp:
	cp mortal/config-offline.toml mortal/config.toml
	cp mortal/model/grp-v0.pth mortal/model/grp.pth
	(cd mortal && python train_grp.py)

offline:
	cp mortal/config-offline.toml mortal/config.toml
	cp mortal/model/grp-v0.pth mortal/model/grp.pth
	(cd mortal && python train.py)

rust:
	# rustup default nightly
	RUSTFLAGS=-g cargo build -p libriichi --lib --release
	cargo build -p libriichi --bins --no-default-features --release
	cargo build -p exe-wrapper
	cp target/release/validate_logs mortal/dataset/
	cp target/release/stat mortal/dataset/
	cp target/release/libriichi.so mortal/
	cp target/release/validate_aka mortal/dataset/

pytorch-pip-cpu:
	pip install torch torchvision torchaudio --extra-index-url https://download.pytorch.org/whl/cpu
	pip install tensorboard

prepare:
	mkdir -p mortal/log/train_play
	mkdir -p mortal/log/test_play
	mkdir -p mortal/log/tensorboard
	mkdir -p mortal/log/buffer
	mkdir -p mortal/log/drain
	mkdir -p mortal/log/1v3
	mkdir -p mortal/model
	mkdir -p mortal/dataset

clean:
	rm mortal/log/buffer/* || true
	rm mortal/log/drain/* || true
	rm mortal/dataset/file_index.pth || true
	rm mortal/dataset/grp_file_index.pth || true

restore:
	cp mortal/model/mortal-v2.1.pth mortal/model/mortal.pth

test:
	(cd mortal && python mortal.py 1 < test.json)

tensorboard:
	tensorboard --logdir mortal/log/tensorboard

conda-python310:
	conda install -c anaconda python=3.10

# ls mjlog_pf4-20_n22 -1 | cut -c -31 | xargs -n 1 -I{} sh -c "./mjai-reviewer --no-review -t {} --mjai-out - | gzip > tenhou_rawlog/{}.json.gz "
