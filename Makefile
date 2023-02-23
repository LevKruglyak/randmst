prog := randmst

debug ?=

ifdef debug
  release :=
  target :=debug
  extension :=debug
else
  release :=--release
  target :=release
  extension :=
endif

all:
	cargo build $(release)
	cp target/$(target)/$(prog) $(prog)
