CC=gcc
DIR=./bin

all: $(DIR) buffer_buffoonery.o0 buffer_buffoonery.o1 buffer_buffoonery.o2 buffer_buffoonery.o3 buffer_buffoonery.s0 buffer_buffoonery.s1 buffer_buffoonery.s2 buffer_buffoonery.s3

$(DIR):
	mkdir -p $(DIR)

buffer_buffoonery.o0: buffer_buffoonery.c
	$(CC) -O0 -o $(DIR)/buffer_buffoonery.o0 buffer_buffoonery.c

buffer_buffoonery.o1: buffer_buffoonery.c
	$(CC) -O1 -o $(DIR)/buffer_buffoonery.o1 buffer_buffoonery.c

buffer_buffoonery.o2: buffer_buffoonery.c
	$(CC) -O2 -o $(DIR)/buffer_buffoonery.o2 buffer_buffoonery.c

buffer_buffoonery.o3: buffer_buffoonery.c
	$(CC) -O3 -o $(DIR)/buffer_buffoonery.o3 buffer_buffoonery.c

buffer_buffoonery.s0: buffer_buffoonery.c
	$(CC) -O0 -S -o $(DIR)/buffer_buffoonery.s0 buffer_buffoonery.c

buffer_buffoonery.s1: buffer_buffoonery.c
	$(CC) -O1 -S -o $(DIR)/buffer_buffoonery.s1 buffer_buffoonery.c

buffer_buffoonery.s2: buffer_buffoonery.c
	$(CC) -O2 -S -o $(DIR)/buffer_buffoonery.s2 buffer_buffoonery.c

buffer_buffoonery.s3: buffer_buffoonery.c
	$(CC) -O3 -S -o $(DIR)/buffer_buffoonery.s3 buffer_buffoonery.c

clean:
	rm -rf $(DIR)/*
