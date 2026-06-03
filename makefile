# makefile for OpenBench
all:
	cargo build -r
	cp target/release/ratsu $(EXE)