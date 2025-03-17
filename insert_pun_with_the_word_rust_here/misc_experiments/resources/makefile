CC=gcc
DIR=./bin
FILENAME=misc_experiments


all: $(DIR) $(FILENAME).o0 $(FILENAME).o1 $(FILENAME).o2 $(FILENAME).o3 $(FILENAME).s0 $(FILENAME).s1 $(FILENAME).s2 $(FILENAME).s3

$(DIR):
	mkdir -p $(DIR)

$(FILENAME).o0: $(FILENAME).c
	$(CC) -O0 -o $(DIR)/$(FILENAME).o0 $(FILENAME).c

$(FILENAME).o1: $(FILENAME).c
	$(CC) -O1 -o $(DIR)/$(FILENAME).o1 $(FILENAME).c

$(FILENAME).o2: $(FILENAME).c
	$(CC) -O2 -o $(DIR)/$(FILENAME).o2 $(FILENAME).c

$(FILENAME).o3: $(FILENAME).c
	$(CC) -O3 -o $(DIR)/$(FILENAME).o3 $(FILENAME).c

$(FILENAME).s0: $(FILENAME).c
	$(CC) -O0 -S -o $(DIR)/$(FILENAME).s0 $(FILENAME).c

$(FILENAME).s1: $(FILENAME).c
	$(CC) -O1 -S -o $(DIR)/$(FILENAME).s1 $(FILENAME).c

$(FILENAME).s2: $(FILENAME).c
	$(CC) -O2 -S -o $(DIR)/$(FILENAME).s2 $(FILENAME).c

$(FILENAME).s3: $(FILENAME).c
	$(CC) -O3 -S -o $(DIR)/$(FILENAME).s3 $(FILENAME).c

clean:
	rm -rf $(DIR)/*
